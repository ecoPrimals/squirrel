impl SyncManager {
    /// Create a new sync manager
    pub fn new(
        context: Arc<RwLock<MachineContext>>,
        config: Option<SyncConfig>,
    ) -> Result<Self, MCPError> {
        let (event_tx, _) = broadcast::channel(100);
        
        Ok(Self {
            context,
            config: config.unwrap_or_default(),
            event_tx,
        })
    }

    /// Start the sync manager
    pub async fn start(&mut self) -> Result<(), MCPError> {
        let event_tx = self.event_tx.clone();
        let context = self.context.clone();
        let config = self.config.clone();

        // Spawn sync event handler
        tokio::spawn({
            let event_tx = event_tx.clone();
            let context = context.clone();
            async move {
                let mut rx = event_tx.subscribe();
                
                while let Ok(event) = rx.recv().await {
                    match event {
                        SyncEvent::ContextUpdate(snapshot) => {
                            if let Ok(mut ctx) = context.write() {
                                if let Err(e) = ctx.restore_from_snapshot(&snapshot) {
                                    let _ = event_tx.send(SyncEvent::Error(e.to_string()));
                                }
                            }
                        },
                        SyncEvent::Error(e) => {
                            eprintln!("Sync error: {}", e);
                        },
                    }
                }
            }
        });

        // Spawn periodic sync checker
        Ok(())
    }
} 
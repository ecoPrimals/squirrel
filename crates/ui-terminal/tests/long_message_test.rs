use ui_terminal::app::chat::{ChatApp, ChatResponse};
use ui_terminal::widgets::chat::ChatState;
use std::sync::Arc;
use dashboard_core::service::MockDashboardService;
use tokio::sync::mpsc;

/// Test that the chat app can handle extremely long responses
#[tokio::test]
async fn test_long_message_handling() {
    // Create test dependencies
    let service = Arc::new(MockDashboardService::new());
    let mut app = ChatApp::<MockDashboardService>::new(service);
    
    // Generate a long message (simulating a poem or long response)
    let long_message = generate_long_message(5000); // 5000 chars
    
    // Log the message length to verify
    println!("Testing with message of length: {}", long_message.len());
    
    // Create a response with the long message
    let response = ChatResponse::final_response(long_message.clone());
    let response_id = response.id;
    
    // Add a thinking message first (simulating normal flow)
    app.has_temp_message = true;
    app.state.add_ai_message("Thinking...".to_string());
    
    // Process the long message
    app.process_single_message(response);
    
    // Verify that:
    // 1. Message was not truncated
    // 2. Thinking message was replaced
    
    assert_eq!(app.state.messages.len(), 1, "There should be exactly one message");
    assert_eq!(app.state.messages[0].content.len(), long_message.len(), 
        "Message should not be truncated");
    assert!(!app.has_temp_message, "Thinking message flag should be cleared");
    assert!(app.processed_messages.contains(&response_id), 
        "Message ID should be in processed set");
}

/// Test handling multiple long messages
#[tokio::test]
async fn test_multiple_long_messages() {
    // Create test dependencies
    let service = Arc::new(MockDashboardService::new());
    let mut app = ChatApp::<MockDashboardService>::new(service);
    
    // Add several long messages
    for i in 1..=5 {
        let length = i * 1000; // Increasing lengths
        let message = generate_long_message(length);
        
        // For odd numbered messages, simulate user messages
        if i % 2 == 1 {
            app.state.add_user_message(message);
        } else {
            // For even numbered messages, simulate AI responses
            app.has_temp_message = true;
            app.state.add_ai_message("Thinking...".to_string());
            let response = ChatResponse::final_response(message);
            app.process_single_message(response);
        }
    }
    
    // Verify correct handling
    assert_eq!(app.state.messages.len(), 5, "All messages should be stored");
    
    // Check that message lengths match what we put in
    for i in 0..5 {
        let expected_length = (i + 1) * 1000;
        let actual_length = app.state.messages[i].content.len();
        
        // Allow some variance for user messages which might have prefixes/formatting
        assert!(
            (actual_length as i32 - expected_length as i32).abs() < 50,
            "Message {} length should be approximately {}, got {}",
            i, expected_length, actual_length
        );
    }
}

/// Test exporting and importing conversation history with long messages
#[tokio::test]
async fn test_history_export_import() {
    // Create test dependencies
    let service = Arc::new(MockDashboardService::new());
    let mut app = ChatApp::<MockDashboardService>::new(service);
    
    // Add some messages including a long one
    app.state.add_user_message("Hello".to_string());
    
    let long_message = generate_long_message(3000);
    app.has_temp_message = true;
    app.state.add_ai_message("Thinking...".to_string());
    let response = ChatResponse::final_response(long_message.clone());
    app.process_single_message(response);
    
    app.state.add_user_message("Thank you for the detailed response".to_string());
    
    // Export history
    let history = app.export_conversation_history();
    
    // Create a new app and import history
    let service2 = Arc::new(MockDashboardService::new());
    let mut app2 = ChatApp::<MockDashboardService>::new(service2);
    app2.import_conversation_history(history);
    
    // Verify history was preserved correctly
    assert_eq!(app.state.messages.len(), app2.state.messages.len(), 
        "Message count should match after import");
    
    // Check content of each message
    for i in 0..app.state.messages.len() {
        assert_eq!(
            app.state.messages[i].content, 
            app2.state.messages[i].content,
            "Message content should be preserved exactly"
        );
        assert_eq!(
            app.state.messages[i].is_user, 
            app2.state.messages[i].is_user,
            "Message user flag should be preserved"
        );
    }
    
    // Specific check for the long message
    let long_msg_idx = 1; // Should be the second message
    assert_eq!(
        app2.state.messages[long_msg_idx].content.len(), 
        long_message.len(),
        "Long message should be preserved exactly"
    );
}

/// Test that the chat app can handle a long poem response
#[tokio::test]
async fn test_long_poem_handling() {
    // Create test dependencies
    let service = Arc::new(MockDashboardService::new());
    let mut app = ChatApp::<MockDashboardService>::new(service);
    
    // A long multi-line poem with about 500 words
    let poem = r#"
The Ancient Mariner's Lament
By AI Assistant

Part I

In darkened seas where shadows creep,
Where sailors pray their souls to keep,
A vessel sailed on waters deep,
While stars above did slowly sweep.

The mariners, with hearts of gold,
Faced tempests fierce and winters cold,
Their stories never fully told,
Their courage more than wealth untold.

Among them stood a weathered man,
His face etched deep since time began,
Through countless storms his life had ran,
Now sharing tales as elders can.

"Attend my words," the old man spoke,
His voice like thunder when it broke,
"A burden carried like a yoke,
A memory that still provokes."

Part II

"Our ship was bound for southern climes,
In search of treasures of old times,
With hearts as pure as chapel chimes,
Before I committed the darkest crimes."

"The winds were fair, the skies were clear,
No sign of danger lurking near,
No reason for a sailor's fear,
Until the albatross did appear."

"A graceful bird of purest white,
That followed us both day and night,
A blessed omen in our sight,
That filled our hearts with pure delight."

"But in a moment's foolish pride,
With crossbow placed against my side,
I shot the bird that was our guide,
And watched as it fell to the tide."

Part III

"The crew at first showed great disdain,
They hung the bird around my chain,
A symbol of my deadly stain,
The start of my unending pain."

"But fortune changed as days passed by,
The winds grew still, the sun scorched high,
No cloud or rain in copper sky,
We watched our water stores run dry."

"'Twas then they blamed the albatross,
On me they placed no blame or cross,
But I alone knew of the loss,
That my own hand had been the cause."

"We drifted on that glassy sea,
No wind to set our vessel free,
While thirst claimed men in agony,
Until at last they turned to me."

Part IV

"One by one they dropped like stones,
Their bodies nothing more than bones,
I heard their final gasps and moans,
Now doomed to wander seas alone."

"A phantom ship approached at last,
With spectre crew and death's repast,
They gambled for each soul that passed,
But left me living — outcast."

"For years I drifted, cursed to roam,
Across the seas of salt and foam,
No port or harbor called me home,
Beneath the vast celestial dome."

"Until I learned to love all things,
The creatures small with fins and wings,
From them this lesson still rings:
Respect all life, or sorrow brings."

Part V

"So heed my words, young mariner,
Let kindness be your standard bearer,
Make gentleness your heart's squire,
And never let your pride grow higher."

"For in this world we briefly dwell,
Each creature has its tale to tell,
Treat all with care and treat them well,
Lest you be trapped in living hell."

The old man's tale now reaches end,
His weathered face beginning to mend,
As if this sharing helped transcend,
The curse that time could never bend.

Remember well this sailor's plight,
His burden carried day and night,
Until he saw the sacred light:
All living things deserve our might.
"#;
    
    println!("Testing with poem of length: {}", poem.len());
    
    // Create a response with the poem
    let response = ChatResponse::final_response(poem.to_string());
    
    // Add a thinking message first
    app.has_temp_message = true;
    app.state.add_ai_message("Thinking about your request for a poem...".to_string());
    
    // Process the poem
    app.process_single_message(response);
    
    // Verify the poem was stored correctly
    assert_eq!(app.state.messages.len(), 1, "There should be exactly one message");
    assert_eq!(app.state.messages[0].content, poem, "Poem content should be preserved exactly");
    assert!(!app.has_temp_message, "Thinking message flag should be cleared");
    
    // Check that the poem has all the stanzas
    let stanza_count = app.state.messages[0].content.split("\n\n").count();
    println!("Poem has {} stanzas", stanza_count);
    assert!(stanza_count > 5, "Poem should have multiple stanzas");
}

/// Helper function to generate a long message for testing
fn generate_long_message(length: usize) -> String {
    let base_text = "This is a test of a very long message. It simulates a detailed response that might be received from an AI assistant. ";
    
    let mut result = String::with_capacity(length);
    while result.len() < length {
        result.push_str(base_text);
    }
    
    // Trim to exact length
    result.truncate(length);
    result
} 
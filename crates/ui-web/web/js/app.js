document.addEventListener('DOMContentLoaded', function() {
    // Global variables
    const API_BASE_URL = window.location.origin + '/api';
    const WS_BASE_URL = window.location.origin.replace(/^http/, 'ws') + '/ws';
    let accessToken = localStorage.getItem('access_token');
    let webSocket = null;
    let commandsList = [];
    let jobsList = [];
    let isConnected = false;

    // DOM Elements
    const navLinks = document.querySelectorAll('nav a');
    const sections = document.querySelectorAll('main section');
    const connectionStatus = document.getElementById('connection-status');
    const commandsListContainer = document.getElementById('commands-list');
    const commandSelect = document.getElementById('command-select');
    const paramsTextarea = document.getElementById('command-parameters');
    const executeCommandButton = document.getElementById('execute-command');
    const commandResultContainer = document.getElementById('command-result');
    const resultIdElement = document.getElementById('result-id');
    const resultStatusElement = document.getElementById('result-status');
    const resultContentElement = document.getElementById('result-content');
    const jobsListContainer = document.getElementById('jobs-list');
    const jobDetailsContainer = document.getElementById('job-details');
    const jobIdElement = document.getElementById('job-id');
    const jobStatusElement = document.getElementById('job-status');
    const jobProgressElement = document.getElementById('job-progress');
    const jobProgressTextElement = document.getElementById('job-progress-text');
    const jobCreatedAtElement = document.getElementById('job-created-at');
    const jobResultElement = document.getElementById('job-result');
    const cancelJobButton = document.getElementById('cancel-job');
    const webStatusElement = document.getElementById('web-status');
    const dbStatusElement = document.getElementById('db-status');
    const mcpStatusElement = document.getElementById('mcp-status');
    const uptimeElement = document.getElementById('uptime');
    const systemInfoElement = document.getElementById('system-info');
    const clearLogsButton = document.getElementById('clear-logs');
    const autoScrollToggle = document.getElementById('autoscroll-logs');
    const logsContainer = document.getElementById('logs-container');
    const loginButton = document.getElementById('login-button');
    const loginModal = document.getElementById('login-modal');
    const submitLoginButton = document.getElementById('submit-login');
    const closeModalButton = document.querySelector('.close-modal');
    const cancelModalButton = document.querySelector('.cancel-modal');
    const authStatusText = document.getElementById('auth-status');

    // Initialize the application
    function init() {
        setupEventListeners();
        checkAuthStatus();
        fetchSystemStatus();
        fetchCommands();
        fetchJobs();
        setupWebSocket();
    }

    // Set up event listeners
    function setupEventListeners() {
        // Navigation
        navLinks.forEach(link => {
            link.addEventListener('click', function(e) {
                e.preventDefault();
                const target = this.getAttribute('data-section');
                activateSection(target);
            });
        });

        // Command execution
        if (executeCommandButton) {
            executeCommandButton.addEventListener('click', executeCommand);
        }

        // Command selection change
        if (commandSelect) {
            commandSelect.addEventListener('change', updateParametersPlaceholder);
        }

        // Job cancellation
        if (cancelJobButton) {
            cancelJobButton.addEventListener('click', cancelSelectedJob);
        }

        // Logs
        if (clearLogsButton) {
            clearLogsButton.addEventListener('click', clearLogs);
        }

        // Authentication
        if (loginButton) {
            loginButton.addEventListener('click', showLoginModal);
        }

        if (submitLoginButton) {
            submitLoginButton.addEventListener('click', login);
        }

        if (closeModalButton) {
            closeModalButton.addEventListener('click', hideLoginModal);
        }

        if (cancelModalButton) {
            cancelModalButton.addEventListener('click', hideLoginModal);
        }
    }

    // Activate a section
    function activateSection(sectionId) {
        navLinks.forEach(link => {
            if (link.getAttribute('data-section') === sectionId) {
                link.classList.add('active');
            } else {
                link.classList.remove('active');
            }
        });

        sections.forEach(section => {
            if (section.id === `${sectionId}-section`) {
                section.classList.add('active-section');
            } else {
                section.classList.remove('active-section');
            }
        });
    }

    // Check authentication status
    function checkAuthStatus() {
        if (accessToken) {
            authStatusText.textContent = 'Authenticated';
        } else {
            authStatusText.textContent = 'Not authenticated';
        }
    }

    // Show login modal
    function showLoginModal() {
        loginModal.style.display = 'block';
    }

    // Hide login modal
    function hideLoginModal() {
        loginModal.style.display = 'none';
    }

    // Login function
    function login() {
        const username = document.getElementById('username').value;
        const password = document.getElementById('password').value;

        if (!username || !password) {
            alert('Please enter both username and password');
            return;
        }

        fetch(`${API_BASE_URL}/auth/login`, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json'
            },
            body: JSON.stringify({ username, password })
        })
        .then(response => {
            if (!response.ok) {
                throw new Error('Login failed');
            }
            return response.json();
        })
        .then(data => {
            accessToken = data.token;
            localStorage.setItem('access_token', accessToken);
            hideLoginModal();
            checkAuthStatus();
            setupWebSocket(); // Reconnect with new token
            addLogEntry({ type: 'system', message: 'User authenticated' });
        })
        .catch(error => {
            console.error('Login error:', error);
            alert('Login failed. Please check your credentials.');
        });
    }

    // Fetch available commands
    function fetchCommands() {
        setLoading(commandsListContainer, true);

        fetch(`${API_BASE_URL}/commands`)
            .then(response => {
                if (!response.ok) {
                    throw new Error('Failed to fetch commands');
                }
                return response.json();
            })
            .then(data => {
                commandsList = data.commands || [];
                renderCommandsList();
                populateCommandSelect();
            })
            .catch(error => {
                console.error('Error fetching commands:', error);
                commandsListContainer.innerHTML = '<div class="error-message">Failed to load commands. Please try again later.</div>';
            })
            .finally(() => {
                setLoading(commandsListContainer, false);
            });
    }

    // Render commands list
    function renderCommandsList() {
        if (!commandsListContainer) return;

        if (!commandsList.length) {
            commandsListContainer.innerHTML = '<div class="empty-message">No commands available.</div>';
            return;
        }

        commandsListContainer.innerHTML = '';
        
        commandsList.forEach(command => {
            const commandCard = document.createElement('div');
            commandCard.className = 'command-card';
            commandCard.innerHTML = `
                <h4>${escapeHtml(command.name)}</h4>
                <p>${escapeHtml(command.description || 'No description available')}</p>
                <div>
                    ${command.tags ? command.tags.map(tag => `<span class="tag">${escapeHtml(tag)}</span>`).join('') : ''}
                </div>
            `;
            
            commandCard.addEventListener('click', () => {
                selectCommand(command.id);
            });
            
            commandsListContainer.appendChild(commandCard);
        });
    }

    // Populate command select dropdown
    function populateCommandSelect() {
        if (!commandSelect) return;
        
        commandSelect.innerHTML = '<option value="">Select a command...</option>';
        
        commandsList.forEach(command => {
            const option = document.createElement('option');
            option.value = command.id;
            option.textContent = command.name;
            commandSelect.appendChild(option);
        });
    }

    // Select a command
    function selectCommand(commandId) {
        if (!commandSelect) return;
        
        commandSelect.value = commandId;
        updateParametersPlaceholder();
    }

    // Update parameters placeholder based on selected command
    function updateParametersPlaceholder() {
        if (!commandSelect || !paramsTextarea) return;
        
        const selectedCommandId = commandSelect.value;
        if (!selectedCommandId) {
            paramsTextarea.value = '';
            return;
        }
        
        const selectedCommand = commandsList.find(cmd => cmd.id === selectedCommandId);
        if (!selectedCommand) return;
        
        // Create a template based on command parameters
        let template = {};
        if (selectedCommand.parameters) {
            selectedCommand.parameters.forEach(param => {
                if (param.type === 'string') {
                    template[param.name] = '';
                } else if (param.type === 'number' || param.type === 'integer') {
                    template[param.name] = 0;
                } else if (param.type === 'boolean') {
                    template[param.name] = false;
                } else if (param.type === 'array') {
                    template[param.name] = [];
                } else if (param.type === 'object') {
                    template[param.name] = {};
                }
            });
        }
        
        paramsTextarea.value = JSON.stringify(template, null, 2);
    }

    // Execute a command
    function executeCommand() {
        if (!commandSelect || !paramsTextarea) return;
        
        const commandId = commandSelect.value;
        if (!commandId) {
            alert('Please select a command');
            return;
        }
        
        let params;
        try {
            params = JSON.parse(paramsTextarea.value);
        } catch (e) {
            alert('Invalid JSON parameters');
            return;
        }
        
        commandResultContainer.classList.remove('hidden');
        resultIdElement.textContent = 'Processing...';
        resultStatusElement.textContent = 'Running';
        resultContentElement.textContent = 'Executing command...';
        
        const headers = {
            'Content-Type': 'application/json'
        };
        
        if (accessToken) {
            headers['Authorization'] = `Bearer ${accessToken}`;
        }
        
        fetch(`${API_BASE_URL}/commands/execute/${commandId}`, {
            method: 'POST',
            headers: headers,
            body: JSON.stringify({ params })
        })
        .then(response => {
            if (!response.ok) {
                throw new Error('Command execution failed');
            }
            return response.json();
        })
        .then(data => {
            resultIdElement.textContent = data.id || 'N/A';
            resultStatusElement.textContent = data.status || 'Unknown';
            resultContentElement.textContent = JSON.stringify(data.result || {}, null, 2);
            
            // Refresh jobs list after execution
            fetchJobs();
            addLogEntry({ type: 'command', message: `Command executed: ${commandId}` });
        })
        .catch(error => {
            console.error('Error executing command:', error);
            resultIdElement.textContent = 'Error';
            resultStatusElement.textContent = 'Failed';
            resultContentElement.textContent = error.message;
            addLogEntry({ type: 'error', message: `Command execution failed: ${error.message}` });
        });
    }

    // Fetch jobs list
    function fetchJobs() {
        if (!jobsListContainer) return;
        
        setLoading(jobsListContainer, true);
        
        const headers = {};
        if (accessToken) {
            headers['Authorization'] = `Bearer ${accessToken}`;
        }
        
        fetch(`${API_BASE_URL}/jobs`, {
            headers: headers
        })
        .then(response => {
            if (!response.ok) {
                throw new Error('Failed to fetch jobs');
            }
            return response.json();
        })
        .then(data => {
            jobsList = data.jobs || [];
            renderJobsList();
        })
        .catch(error => {
            console.error('Error fetching jobs:', error);
            jobsListContainer.innerHTML = '<div class="error-message">Failed to load jobs. Please try again later.</div>';
        })
        .finally(() => {
            setLoading(jobsListContainer, false);
        });
    }

    // Render jobs list
    function renderJobsList() {
        if (!jobsListContainer) return;
        
        if (!jobsList.length) {
            jobsListContainer.innerHTML = '<div class="empty-message">No jobs available.</div>';
            return;
        }
        
        jobsListContainer.innerHTML = '';
        
        jobsList.forEach(job => {
            const jobItem = document.createElement('div');
            jobItem.className = 'job-item';
            jobItem.dataset.jobId = job.id;
            jobItem.innerHTML = `
                <div class="job-name">${escapeHtml(job.name || 'Unknown job')}</div>
                <div class="job-status status-${job.status.toLowerCase()}">${escapeHtml(job.status)}</div>
            `;
            
            jobItem.addEventListener('click', () => {
                // Remove selected class from all jobs
                document.querySelectorAll('.job-item').forEach(item => {
                    item.classList.remove('selected');
                });
                
                // Add selected class to clicked job
                jobItem.classList.add('selected');
                
                // Show job details
                fetchJobDetails(job.id);
            });
            
            jobsListContainer.appendChild(jobItem);
        });
    }

    // Fetch job details
    function fetchJobDetails(jobId) {
        if (!jobDetailsContainer) return;
        
        jobDetailsContainer.classList.remove('hidden');
        
        const headers = {};
        if (accessToken) {
            headers['Authorization'] = `Bearer ${accessToken}`;
        }
        
        fetch(`${API_BASE_URL}/jobs/${jobId}`, {
            headers: headers
        })
        .then(response => {
            if (!response.ok) {
                throw new Error('Failed to fetch job details');
            }
            return response.json();
        })
        .then(job => {
            // Update job details elements
            jobIdElement.textContent = job.id;
            jobStatusElement.textContent = job.status;
            
            // Calculate progress
            const progress = job.progress ? Math.round((job.progress.current / job.progress.total) * 100) : 0;
            jobProgressElement.style.width = `${progress}%`;
            jobProgressTextElement.textContent = `${progress}%`;
            
            // Update dates and result
            jobCreatedAtElement.textContent = job.created_at ? new Date(job.created_at).toLocaleString() : 'Unknown';
            jobResultElement.textContent = JSON.stringify(job.result || {}, null, 2);
            
            // Show or hide cancel button based on job status
            if (job.status === 'RUNNING' || job.status === 'QUEUED') {
                cancelJobButton.style.display = 'block';
            } else {
                cancelJobButton.style.display = 'none';
            }
            
            // Store job ID for cancel button
            cancelJobButton.dataset.jobId = job.id;
        })
        .catch(error => {
            console.error('Error fetching job details:', error);
            jobDetailsContainer.innerHTML = '<div class="error-message">Failed to load job details. Please try again later.</div>';
        });
    }

    // Cancel a job
    function cancelSelectedJob() {
        const jobId = cancelJobButton.dataset.jobId;
        if (!jobId) return;
        
        if (!confirm('Are you sure you want to cancel this job?')) {
            return;
        }
        
        const headers = {
            'Content-Type': 'application/json'
        };
        
        if (accessToken) {
            headers['Authorization'] = `Bearer ${accessToken}`;
        }
        
        fetch(`${API_BASE_URL}/jobs/${jobId}/cancel`, {
            method: 'POST',
            headers: headers
        })
        .then(response => {
            if (!response.ok) {
                throw new Error('Failed to cancel job');
            }
            return response.json();
        })
        .then(data => {
            addLogEntry({ type: 'system', message: `Job ${jobId} cancelled` });
            fetchJobs(); // Refresh jobs list
            fetchJobDetails(jobId); // Refresh job details
        })
        .catch(error => {
            console.error('Error cancelling job:', error);
            alert('Failed to cancel job: ' + error.message);
        });
    }

    // Fetch system status
    function fetchSystemStatus() {
        fetch(`${API_BASE_URL}/status`)
            .then(response => {
                if (!response.ok) {
                    throw new Error('Failed to fetch system status');
                }
                return response.json();
            })
            .then(data => {
                // Update status elements
                webStatusElement.textContent = data.web_status || 'Unknown';
                webStatusElement.className = getStatusClass(data.web_status);
                
                dbStatusElement.textContent = data.db_status || 'Unknown';
                dbStatusElement.className = getStatusClass(data.db_status);
                
                mcpStatusElement.textContent = data.mcp_status || 'Unknown';
                mcpStatusElement.className = getStatusClass(data.mcp_status);
                
                uptimeElement.textContent = data.uptime || 'Unknown';
                
                systemInfoElement.textContent = JSON.stringify(data.system_info || {}, null, 2);
            })
            .catch(error => {
                console.error('Error fetching system status:', error);
                webStatusElement.textContent = 'Error';
                webStatusElement.className = 'status-error';
                
                dbStatusElement.textContent = 'Error';
                dbStatusElement.className = 'status-error';
                
                mcpStatusElement.textContent = 'Error';
                mcpStatusElement.className = 'status-error';
                
                uptimeElement.textContent = 'Error';
                systemInfoElement.textContent = 'Failed to load system information.';
            });
    }

    // Get CSS class for status value
    function getStatusClass(status) {
        if (!status) return 'status-unknown';
        
        const statusLower = status.toLowerCase();
        if (statusLower === 'ok' || statusLower === 'online' || statusLower === 'connected') {
            return 'status-online';
        } else if (statusLower === 'warning' || statusLower === 'partial') {
            return 'status-warning';
        } else if (statusLower === 'error' || statusLower === 'offline' || statusLower === 'disconnected') {
            return 'status-offline';
        } else {
            return 'status-unknown';
        }
    }

    // Set up WebSocket connection
    function setupWebSocket() {
        if (webSocket) {
            webSocket.close();
        }
        
        let wsUrl = WS_BASE_URL;
        if (accessToken) {
            wsUrl += `?token=${accessToken}`;
        }
        
        webSocket = new WebSocket(wsUrl);
        
        webSocket.onopen = function() {
            isConnected = true;
            updateConnectionStatus(true);
            addLogEntry({ type: 'system', message: 'WebSocket connected' });
        };
        
        webSocket.onclose = function() {
            isConnected = false;
            updateConnectionStatus(false);
            addLogEntry({ type: 'system', message: 'WebSocket disconnected' });
            
            // Try to reconnect after 5 seconds
            setTimeout(setupWebSocket, 5000);
        };
        
        webSocket.onerror = function(error) {
            console.error('WebSocket error:', error);
            isConnected = false;
            updateConnectionStatus(false);
            addLogEntry({ type: 'error', message: 'WebSocket error' });
        };
        
        webSocket.onmessage = function(event) {
            handleWebSocketMessage(event.data);
        };
    }

    // Update connection status
    function updateConnectionStatus(connected) {
        if (!connectionStatus) return;
        
        if (connected) {
            connectionStatus.textContent = 'Online';
            connectionStatus.className = 'status-online';
        } else {
            connectionStatus.textContent = 'Offline';
            connectionStatus.className = 'status-offline';
        }
    }

    // Handle WebSocket messages
    function handleWebSocketMessage(data) {
        try {
            const message = JSON.parse(data);
            
            // Log the message
            addLogEntry({ type: 'event', message: JSON.stringify(message) });
            
            // Handle message types
            switch (message.type) {
                case 'job_update':
                    // Refresh jobs list when a job is updated
                    fetchJobs();
                    break;
                case 'system_update':
                    // Update system info when received
                    fetchSystemStatus();
                    break;
                case 'command_update':
                    // Refresh commands list when commands are updated
                    fetchCommands();
                    break;
            }
        } catch (error) {
            console.error('Error handling WebSocket message:', error);
            addLogEntry({ type: 'error', message: 'Failed to parse WebSocket message' });
        }
    }

    // Add log entry
    function addLogEntry(data) {
        if (!logsContainer) return;
        
        const logEntry = document.createElement('div');
        const time = new Date().toLocaleTimeString();
        
        let logClass = 'system-log';
        
        if (data.type === 'command') {
            logClass = 'command-log';
        } else if (data.type === 'event') {
            logClass = 'event-log';
        } else if (data.type === 'error') {
            logClass = 'error-log';
        }
        
        logEntry.className = `log-entry ${logClass}`;
        logEntry.innerHTML = `
            <span class="log-time">${time}</span>
            <span class="log-content">${escapeHtml(data.message)}</span>
        `;
        
        logsContainer.appendChild(logEntry);
        
        // Auto-scroll to bottom if enabled
        if (autoScrollToggle && autoScrollToggle.checked) {
            logsContainer.scrollTop = logsContainer.scrollHeight;
        }
    }

    // Clear logs
    function clearLogs() {
        if (!logsContainer) return;
        
        if (confirm('Are you sure you want to clear all logs?')) {
            logsContainer.innerHTML = '';
            addLogEntry({ type: 'system', message: 'Logs cleared' });
        }
    }

    // Helper function to set loading state
    function setLoading(element, isLoading) {
        if (!element) return;
        
        if (isLoading) {
            const loadingElement = document.createElement('div');
            loadingElement.className = 'loading';
            loadingElement.textContent = 'Loading...';
            element.innerHTML = '';
            element.appendChild(loadingElement);
        } else {
            const loadingElement = element.querySelector('.loading');
            if (loadingElement) {
                loadingElement.remove();
            }
        }
    }

    // Helper function to escape HTML
    function escapeHtml(unsafe) {
        if (unsafe === null || unsafe === undefined) return '';
        if (typeof unsafe !== 'string') {
            unsafe = String(unsafe);
        }
        return unsafe
            .replace(/&/g, '&amp;')
            .replace(/</g, '&lt;')
            .replace(/>/g, '&gt;')
            .replace(/"/g, '&quot;')
            .replace(/'/g, '&#039;');
    }

    // Initialize the application
    init();

    // Set default active section
    activateSection('commands');
}); 
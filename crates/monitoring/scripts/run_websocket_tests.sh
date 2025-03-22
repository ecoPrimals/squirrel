#!/bin/bash
# Enhanced WebSocket Test Runner
# This script runs the WebSocket test and generates a performance report

# Default configuration parameters
NUM_CLIENTS=20
TEST_DURATION_SECONDS=180
REPORT_PATH="./websocket_test_report.md"
GENERATE_CHART=true
VERBOSE_OUTPUT=false

# Parse command line arguments
while [[ $# -gt 0 ]]; do
  case $1 in
    --num-clients)
      NUM_CLIENTS="$2"
      shift 2
      ;;
    --duration)
      TEST_DURATION_SECONDS="$2"
      shift 2
      ;;
    --report)
      REPORT_PATH="$2"
      shift 2
      ;;
    --no-chart)
      GENERATE_CHART=false
      shift
      ;;
    --verbose)
      VERBOSE_OUTPUT=true
      shift
      ;;
    *)
      echo "Unknown option: $1"
      exit 1
      ;;
  esac
done

# Display test configuration
echo "=================================="
echo "WebSocket Test Configuration"
echo "=================================="
echo "Number of Clients: $NUM_CLIENTS"
echo "Test Duration: $TEST_DURATION_SECONDS seconds"
echo "Report Path: $REPORT_PATH"
echo "Generate Chart: $GENERATE_CHART"
echo "Verbose Output: $VERBOSE_OUTPUT"
echo "=================================="

# Check if server is running
if nc -z localhost 8765 &>/dev/null; then
  echo "✅ WebSocket server is running on port 8765"
else
  echo "❌ WebSocket server is not running on port 8765"
  
  # Ask if user wants to start server
  read -p "Do you want to start the WebSocket server? (y/n) " START_SERVER
  if [[ "$START_SERVER" == "y" ]]; then
    echo "Starting WebSocket server in background..."
    cargo run --example dashboard_server &
    SERVER_PID=$!
    
    echo "Waiting for server to start..."
    sleep 5
  else
    echo "Exiting..."
    exit 1
  fi
fi

# Create output directory if it doesn't exist
OUTPUT_DIR=$(dirname "$REPORT_PATH")
mkdir -p "$OUTPUT_DIR"

# Get timestamp for the report
TIMESTAMP=$(date "+%Y-%m-%d %H:%M:%S")

# Run the WebSocket test with specified parameters
echo "Starting WebSocket test with $NUM_CLIENTS clients for $TEST_DURATION_SECONDS seconds..."
TEST_OUTPUT=$(cargo run --example enhanced_websocket_test 2>&1)

# Save raw output to file
RAW_OUTPUT_PATH="${REPORT_PATH%.md}_raw.txt"
echo "$TEST_OUTPUT" > "$RAW_OUTPUT_PATH"

# Extract test statistics
CONNECTIONS_SUCCESSFUL=$(echo "$TEST_OUTPUT" | grep "Connections successful:" | sed -E 's/Connections successful: ([0-9]+)/\1/')
CONNECTION_FAILURES=$(echo "$TEST_OUTPUT" | grep "Connection failures:" | sed -E 's/Connection failures: ([0-9]+)/\1/')
CONNECTIONS_CLOSED=$(echo "$TEST_OUTPUT" | grep "Connections closed:" | sed -E 's/Connections closed: ([0-9]+)/\1/')
TOTAL_MESSAGES=$(echo "$TEST_OUTPUT" | grep "Total messages received:" | sed -E 's/Total messages received: ([0-9]+)/\1/')
RECONNECTION_TESTS=$(echo "$TEST_OUTPUT" | grep "Reconnection tests:" | sed -E 's/Reconnection tests: ([0-9]+)/\1/')

# Extract messages per component
CAPTURE_COMPONENT=false
MESSAGES_PER_COMPONENT=""

# Extract client statistics
CAPTURE_CLIENT=false
CLIENT_STATS=""

# Process the output line by line
while IFS= read -r line; do
  # Check for section markers
  if [[ "$line" == *"--- Messages Per Component ---"* ]]; then
    CAPTURE_COMPONENT=true
    CAPTURE_CLIENT=false
    continue
  elif [[ "$line" == *"--- Client Statistics ---"* ]]; then
    CAPTURE_COMPONENT=false
    CAPTURE_CLIENT=true
    continue
  elif [[ "$line" == *"============================"* ]]; then
    CAPTURE_COMPONENT=false
    CAPTURE_CLIENT=false
    continue
  fi
  
  # Capture messages per component
  if [[ "$CAPTURE_COMPONENT" == true && "$line" =~ ([^:]+):\ ([0-9]+) ]]; then
    COMPONENT="${BASH_REMATCH[1]}"
    COUNT="${BASH_REMATCH[2]}"
    MESSAGES_PER_COMPONENT+="| $COMPONENT | $COUNT |\n"
  fi
  
  # Capture client statistics
  if [[ "$CAPTURE_CLIENT" == true && "$line" =~ client-([0-9]+):\ ([0-9]+)\ attempts,\ ([0-9]+)\ messages,\ ([0-9]+)\ subscriptions ]]; then
    CLIENT_ID="${BASH_REMATCH[1]}"
    ATTEMPTS="${BASH_REMATCH[2]}"
    MESSAGES="${BASH_REMATCH[3]}"
    SUBSCRIPTIONS="${BASH_REMATCH[4]}"
    CLIENT_STATS+="| client-$CLIENT_ID | $ATTEMPTS | $MESSAGES | $SUBSCRIPTIONS |\n"
  fi
done <<< "$TEST_OUTPUT"

# Calculate performance metrics
if [[ -n "$CLIENT_STATS" ]]; then
  NUM_CLIENTS_ACTUAL=$(echo "$CLIENT_STATS" | wc -l)
else
  NUM_CLIENTS_ACTUAL=0
fi

if [[ "$NUM_CLIENTS_ACTUAL" -gt 0 ]]; then
  AVG_MESSAGES_PER_CLIENT=$(echo "scale=2; $TOTAL_MESSAGES / $NUM_CLIENTS_ACTUAL" | bc)
else
  AVG_MESSAGES_PER_CLIENT=0
fi

if [[ "$CONNECTIONS_SUCCESSFUL" -gt 0 || "$CONNECTION_FAILURES" -gt 0 ]]; then
  CONNECTION_SUCCESS_RATE=$(echo "scale=2; ($CONNECTIONS_SUCCESSFUL * 100) / ($CONNECTIONS_SUCCESSFUL + $CONNECTION_FAILURES)" | bc)
else
  CONNECTION_SUCCESS_RATE=0
fi

MESSAGES_PER_SECOND=$(echo "scale=2; $TOTAL_MESSAGES / $TEST_DURATION_SECONDS" | bc)

# Generate conclusion based on results
if (( $(echo "$MESSAGES_PER_SECOND > 50" | bc -l) )) && (( $(echo "$CONNECTION_SUCCESS_RATE > 95" | bc -l) )); then
  CONCLUSION="The WebSocket server performed **excellently** under load, handling multiple concurrent clients with high message throughput and reliable reconnections."
elif (( $(echo "$MESSAGES_PER_SECOND > 20" | bc -l) )) && (( $(echo "$CONNECTION_SUCCESS_RATE > 85" | bc -l) )); then
  CONCLUSION="The WebSocket server performed **well** under load, with good message throughput and mostly reliable reconnections."
else
  CONCLUSION="The WebSocket server performance **needs improvement**, particularly in handling message throughput and connection reliability."
fi

# Generate Markdown report
cat > "$REPORT_PATH" << EOL
# WebSocket Test Report

## Test Configuration
- **Date**: $TIMESTAMP
- **Number of Clients**: $NUM_CLIENTS
- **Test Duration**: $TEST_DURATION_SECONDS seconds
- **Server Address**: ws://localhost:8765/ws

## Test Results Summary

| Metric | Value |
|--------|-------|
| Connections Successful | $CONNECTIONS_SUCCESSFUL |
| Connection Failures | $CONNECTION_FAILURES |
| Connections Closed | $CONNECTIONS_CLOSED |
| Total Messages Received | $TOTAL_MESSAGES |
| Reconnection Tests | $RECONNECTION_TESTS |

## Messages Per Component

| Component | Message Count |
|-----------|--------------|
$(echo -e "$MESSAGES_PER_COMPONENT")

## Client Performance

| Client | Connection Attempts | Messages Received | Components Subscribed |
|--------|---------------------|-------------------|----------------------|
$(echo -e "$CLIENT_STATS")

## Performance Analysis

| Metric | Value |
|--------|-------|
| Messages Per Second | $MESSAGES_PER_SECOND |
| Average Messages Per Client | $AVG_MESSAGES_PER_CLIENT |
| Connection Success Rate | ${CONNECTION_SUCCESS_RATE}% |

## Reconnection Performance

The test included $RECONNECTION_TESTS simulated disconnections to test the reconnection logic.
The connection success rate of ${CONNECTION_SUCCESS_RATE}% indicates the reliability of the reconnection mechanism.

## Conclusion

$CONCLUSION

## Raw Test Output

See the complete test output in the file: $RAW_OUTPUT_PATH

EOL

# Add charts if enabled
if [[ "$GENERATE_CHART" == true ]]; then
  echo "## Performance Charts" >> "$REPORT_PATH"
  echo "" >> "$REPORT_PATH"
  echo "### Messages Per Component" >> "$REPORT_PATH"
  echo "\`\`\`mermaid" >> "$REPORT_PATH"
  echo "pie title Messages Per Component" >> "$REPORT_PATH"
  
  # Extract component data for chart
  while IFS= read -r line; do
    if [[ "$line" =~ \|\ ([^|]+)\|\ ([0-9]+)\ \| ]]; then
      COMPONENT="${BASH_REMATCH[1]}"
      COUNT="${BASH_REMATCH[2]}"
      echo "    \"$COMPONENT\" : $COUNT" >> "$REPORT_PATH"
    fi
  done <<< "$MESSAGES_PER_COMPONENT"
  
  echo "\`\`\`" >> "$REPORT_PATH"
  echo "" >> "$REPORT_PATH"
  
  echo "### Client Message Distribution" >> "$REPORT_PATH"
  echo "\`\`\`mermaid" >> "$REPORT_PATH"
  echo "bar" >> "$REPORT_PATH"
  echo "    title Client Message Distribution" >> "$REPORT_PATH"
  echo "    x-axis [Clients]" >> "$REPORT_PATH"
  echo "    y-axis [Message Count]" >> "$REPORT_PATH"
  
  # Extract client data for chart
  while IFS= read -r line; do
    if [[ "$line" =~ \|\ (client-[0-9]+)\ \|\ [0-9]+\ \|\ ([0-9]+)\ \| ]]; then
      CLIENT="${BASH_REMATCH[1]}"
      MESSAGES="${BASH_REMATCH[2]}"
      echo "    \"$CLIENT\" : $MESSAGES" >> "$REPORT_PATH"
    fi
  done <<< "$CLIENT_STATS"
  
  echo "\`\`\`" >> "$REPORT_PATH"
fi

echo "=================================="
echo "Test completed!"
echo "Report saved to: $REPORT_PATH"
echo "Raw output saved to: $RAW_OUTPUT_PATH"
echo "=================================="

# Clean up server process if we started it
if [[ -n "$SERVER_PID" ]]; then
  echo "Stopping WebSocket server..."
  kill $SERVER_PID 2>/dev/null || true
fi

# Open the report if verbose output is enabled
if [[ "$VERBOSE_OUTPUT" == true ]]; then
  echo "Opening report..."
  if [[ "$OSTYPE" == "darwin"* ]]; then
    open "$REPORT_PATH"
  elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
    xdg-open "$REPORT_PATH" &>/dev/null || true
  fi
fi 
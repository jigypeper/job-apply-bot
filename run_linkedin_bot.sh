#!/bin/bash

# Check if required arguments are provided
if [ $# -lt 2 ]; then
    echo "Usage: $0 <path_to_rust_binary> <linkedin_job_url> [max_applications]"
    echo "Example: $0 /home/user/linkedin-bot/target/release/linkedin-bot https://www.linkedin.com/jobs/search/?keywords=rust%20developer 5"
    exit 1
fi

# Parse arguments
RUST_BINARY="$1"
JOB_URL="$2"
MAX_APPLICATIONS="${3:-5}"  # Default to 5 if not specified

# Check if binary exists
if [ ! -f "$RUST_BINARY" ]; then
    echo "Error: Binary not found at $RUST_BINARY"
    exit 1
fi

# Log file with timestamp
LOG_DIR="$HOME/linkedin_bot_logs"
mkdir -p "$LOG_DIR"
LOG_FILE="$LOG_DIR/linkedin_bot_$(date +%Y%m%d_%H%M%S).log"

echo "Starting LinkedIn job application bot at $(date)" | tee -a "$LOG_FILE"
echo "Job URL: $JOB_URL" | tee -a "$LOG_FILE"
echo "Max applications: $MAX_APPLICATIONS" | tee -a "$LOG_FILE"

# Run the bot
"$RUST_BINARY" "$JOB_URL" --max-applications "$MAX_APPLICATIONS" 2>&1 | tee -a "$LOG_FILE"

EXIT_CODE=${PIPESTATUS[0]}
echo "Bot finished with exit code $EXIT_CODE at $(date)" | tee -a "$LOG_FILE"

exit $EXIT_CODE

# example crons
# # Monday: Apply for Rust Developer jobs
# 0 9 * * 1 /path/to/run_linkedin_bot.sh /path/to/target/release/job-apply-bot "https://www.linkedin.com/jobs/search/?keywords=rust%20developer" 3

# Wednesday: Apply for Backend Engineer jobs
# 0 10 * * 3 /path/to/run_linkedin_bot.sh /path/to/target/release/job-apply-bot "https://www.linkedin.com/jobs/search/?keywords=backend%20engineer" 5

# Friday: Apply for Systems Engineer jobs
# 0 11 * * 5 /path/to/run_linkedin_bot.sh /path/to/target/release/job-apply-bot "https://www.linkedin.com/jobs/search/?keywords=systems%20engineer" 4

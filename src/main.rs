use clap::Parser;
use dotenv::dotenv;
use rand::prelude::IndexedRandom;
use rand::Rng;
use std::env;
use std::time::Duration;
use tempfile::TempDir;
use thirtyfour::prelude::*;
use tokio::time::sleep;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The LinkedIn job search URL
    job_url: String,

    /// Maximum number of applications to submit per run
    #[arg(short, long, default_value = "5")]
    max_applications: usize,
}

// Function to create random delays between actions
async fn random_delay(min_secs: u64, max_secs: u64) {
    let delay = rand::rng().random_range(min_secs..=max_secs);
    println!("Waiting for {} seconds...", delay);
    sleep(Duration::from_secs(delay)).await;
}

// Function to scroll the page randomly to mimic human behavior
async fn scroll_randomly(driver: &WebDriver) -> WebDriverResult<()> {
    let script = "window.scrollBy(0, arguments[0]);";
    let scroll_amount = rand::rng().random_range(100..500);
    driver.execute(script, vec![scroll_amount.into()]).await?;
    println!("Scrolled page by {} pixels", scroll_amount);
    Ok(())
}

#[tokio::main]
async fn main() -> WebDriverResult<()> {
    dotenv().ok(); // Load environment variables from .env file

    let linkedin_id = env::var("LINKEDIN_ID").expect("Missing LINKEDIN_ID");
    let linkedin_key = env::var("LINKEDIN_KEY").expect("Missing LINKEDIN_KEY");

    let args = Args::parse();
    let job_url = args.job_url;
    let max_applications = args.max_applications;

    println!("Starting job application bot...");
    println!("Job search URL: {}", job_url);
    println!("Maximum applications: {}", max_applications);

    // Set up realistic user agents
    let user_agents = [
        "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/100.0.4896.75 Safari/537.36",
        "Mozilla/5.0 (X11; Ubuntu; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/99.0.4844.51 Safari/537.36",
        "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/101.0.4951.64 Safari/537.36",
        "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/102.0.5005.61 Safari/537.36",
        "Mozilla/5.0 (X11; Fedora; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/100.0.4896.127 Safari/537.36"
    ];

    // Randomly select a user agent
    let user_agent = user_agents.choose(&mut rand::rng()).unwrap();
    println!("Using user agent: {}", user_agent);

    // Set up Chromium with headless mode
    let mut caps = DesiredCapabilities::chrome();
    caps.set_headless()?;
    caps.add_arg("--disable-gpu")?;
    caps.add_arg("--no-sandbox")?;
    caps.add_arg("--disable-dev-shm-usage")?;
    caps.add_arg(format!("--user-agent={}", user_agent).as_str())?;

    // Set a unique user data directory to avoid conflicts
    let tmp_dir = TempDir::new()?;
    let user_data_dir = tmp_dir.path().to_str().unwrap();
    caps.add_arg(format!("--user-data-dir={}", user_data_dir).as_str())?;

    println!("Connecting to WebDriver...");
    let driver = WebDriver::new("http://localhost:9515", caps).await?;

    println!("Connected successfully. Opening LinkedIn...");
    // Open LinkedIn Jobs page
    driver.get(&job_url).await?;
    random_delay(5, 10).await;

    // Click 'Sign in'
    match driver.find(By::LinkText("Sign in")).await {
        Ok(login_button) => {
            println!("Found sign-in button, clicking...");
            login_button.click().await?;
            random_delay(2, 5).await;
        }
        Err(e) => {
            println!("Sign-in button not found, may already be logged in: {}", e);
            // Continue as we might already be logged in
        }
    }

    // Check if we need to log in
    if let Ok(username_field) = driver.find(By::Css("#username")).await {
        println!("Logging in...");
        // Enter credentials
        username_field.send_keys(&linkedin_id).await?;
        random_delay(1, 3).await;

        driver
            .find(By::Css("#password"))
            .await?
            .send_keys(&linkedin_key)
            .await?;
        random_delay(1, 3).await;

        driver
            .find(By::Css("#password"))
            .await?
            .send_keys(Key::Enter)
            .await?;
        println!("Login credentials submitted, waiting for page load...");
        random_delay(7, 12).await; // Allow login to complete
    } else {
        println!("Already logged in, continuing...");
    }

    // Scroll down a bit to load more content
    scroll_randomly(&driver).await?;
    random_delay(2, 5).await;

    // Get job listings
    println!("Finding job listings...");
    let job_listings = driver
        .query(By::Css(".job-card-list__title"))
        .all_from_selector()
        .await?;

    println!("Found {} job listings.", job_listings.len());

    // Randomly select a subset of jobs to apply for
    let mut rng = rand::rng();
    let max_to_process = std::cmp::min(job_listings.len(), max_applications * 3); // Process more than max applications to account for skips
    let selected_indices: Vec<usize> = (0..job_listings.len()).collect();
    let selected_indices = selected_indices
        .choose_multiple(&mut rng, max_to_process)
        .cloned()
        .collect::<Vec<_>>();

    println!("Selected {} jobs to process", selected_indices.len());

    let mut application_count = 0;

    for &index in &selected_indices {
        if application_count >= max_applications {
            println!("Reached maximum application limit of {}", max_applications);
            break;
        }

        // Get the job listing again as references may become stale
        let job_listings = driver
            .query(By::Css(".job-card-list__title"))
            .all_from_selector()
            .await?;

        if index >= job_listings.len() {
            println!("Job index {} is no longer valid, skipping", index);
            continue;
        }

        let listing: &WebElement = &job_listings[index];

        // Get the job title for logging
        let job_title = listing
            .text()
            .await
            .unwrap_or_else(|_| "Unknown job title".to_string());
        println!("Processing job: {}", job_title);

        // Sometimes view job details without applying (more human-like behavior)
        if rng.random_bool(0.2) && application_count > 0 {
            // 20% chance, but not for the first job
            println!("Just viewing this job without applying");
            listing.click().await?;
            random_delay(5, 15).await;
            scroll_randomly(&driver).await?;
            random_delay(3, 8).await;
            continue;
        }

        // Click on the job to view details
        listing.click().await?;
        random_delay(3, 7).await;

        // Scroll down to view job details
        scroll_randomly(&driver).await?;
        random_delay(2, 5).await;

        // Sometimes check the company details (more human-like behavior)
        if rng.random_bool(0.3) {
            // 30% chance
            if let Ok(company_link) = driver
                .find(By::Css(".jobs-unified-top-card__company-name"))
                .await
            {
                println!("Checking company profile...");
                company_link.click().await?;
                random_delay(5, 15).await;
                scroll_randomly(&driver).await?;
                random_delay(3, 7).await;
                driver.back().await?;
                random_delay(3, 6).await;
            }
        }

        // Try to find the "Easy Apply" button
        match driver.find(By::Css(".jobs-s-apply button")).await {
            Ok(apply_button) => {
                println!("Found 'Easy Apply' button, clicking...");
                apply_button.click().await?;
                random_delay(4, 7).await;

                // Check if it's a single-click apply or multi-step
                match driver.find(By::Css("footer button")).await {
                    Ok(submit_button) => {
                        let button_text = submit_button.text().await?;
                        if button_text.contains("Next") {
                            // Multi-step application, skip it
                            println!("This is a multi-step application, skipping...");
                            if let Ok(close_button) =
                                driver.find(By::ClassName("artdeco-modal__dismiss")).await
                            {
                                close_button.click().await?;
                                random_delay(1, 3).await;

                                // Handle multiple discard buttons properly
                                if let Ok(discard_buttons) = driver
                                    .query(By::ClassName("artdeco-modal__confirm-dialog-btn"))
                                    .all_from_selector()
                                    .await
                                {
                                    if discard_buttons.len() > 1 {
                                        if let Some(discard_button) = discard_buttons.get(1) {
                                            discard_button.click().await?;
                                        }
                                    } else if let Some(first_discard) = discard_buttons.first() {
                                        first_discard.click().await?;
                                    }
                                }
                            }
                            random_delay(2, 4).await;
                        } else {
                            // Single click apply
                            println!("This is a single-click application, submitting...");
                            submit_button.click().await?;
                            application_count += 1;
                            println!(
                                "Successfully applied to job! ({}/{})",
                                application_count, max_applications
                            );
                            random_delay(5, 10).await;
                        }
                    }
                    Err(e) => {
                        println!("Could not find submit button: {}", e);
                        // Try to close any open modals
                        if let Ok(close_button) =
                            driver.find(By::ClassName("artdeco-modal__dismiss")).await
                        {
                            close_button.click().await?;
                            random_delay(2, 4).await;
                        }
                    }
                }

                // Close any popup after applying
                if let Ok(close_button) = driver.find(By::ClassName("artdeco-modal__dismiss")).await
                {
                    close_button.click().await?;
                    random_delay(2, 4).await;
                }
            }
            Err(_) => {
                println!("No 'Easy Apply' button found, skipping this job.");
                random_delay(2, 4).await;
            }
        }

        // Add a longer delay between job applications
        random_delay(10, 30).await;
    }

    println!(
        "Bot session complete! Applied to {} jobs out of {} maximum.",
        application_count, max_applications
    );
    driver.quit().await?;
    Ok(())
}

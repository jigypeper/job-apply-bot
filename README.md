# Job Application Bot (Rust)

This project is a **job application automation bot** built using **Rust** and the **thirtyfour** crate (Rust binding for Selenium WebDriver). The bot automates applying for jobs on LinkedIn using a given job search URL.

## Features

- **Automates job applications on LinkedIn** based on a given search URL.
- **Supports headless browser automation** (using Chrome or Firefox).
- **Configurable** via CLI arguments to provide a dynamic job search URL.

## Requirements

Before running this bot, you need the following:

- **Rust**: The bot is built using Rust. You'll need to install Rust on your machine if you don't have it already.
- **WebDriver (ChromeDriver/GeckoDriver)**: The bot interacts with a browser via a WebDriver. You need to install either `chromedriver` (for Chrome) or `geckodriver` (for Firefox).

### **1. Install Rust**

If you haven't installed Rust, you can do so by following the instructions here:  
[Rust Installation Guide](https://www.rust-lang.org/tools/install)

### **2. Install WebDriver**

You need to install a WebDriver that the bot will interact with (either Chrome or Firefox).

#### **For Chrome (ChromeDriver)**

1. Install `chromedriver`:
   ```bash
   sudo apt install chromedriver

use std::env;
use std::fmt::Display;
use std::io::{self, Write};

use async_recursion::async_recursion;
use reqwest::header::{HeaderMap, AUTHORIZATION, CONTENT_TYPE};
use reqwest::Client;
use serde::Deserialize;

#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct User {
    id: String,
    display_name: String,
    job_title: Option<String>,
    department: Option<String>,
    mail: Option<String>,
    office_location: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct UsersResponse {
    value: Vec<User>,
    #[serde(rename = "@odata.nextLink")]
    next_link: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let access_token =
        env::var("ACCESS_TOKEN").expect("ACCESS_TOKEN environment variable is not set");
    let search_name = read_input("Enter the display name to search: ")?;
    let filter = format!("startswith(displayName, '{}')", search_name);
    let url = format!("https://graph.microsoft.com/beta/users?$filter={}", filter);

    let client = Client::new();

    let selected_user = loop {
        let response = fetch_users(&client, &access_token, &url).await?;
        let users = response.value;

        if users.is_empty() {
            eprintln!("No users found with the given display name.");
            return Ok(());
        }

        eprintln!("Select a user by entering the index number:");
        for (i, user) in users.iter().enumerate() {
            eprintln!(
                "{}. {} (Email: {})",
                i + 1,
                user.display_name,
                user.get_email()
            );
        }

        let selected_index = read_input("Enter the index of the selected user: ")?;
        let selected_index: usize = selected_index.trim().parse().unwrap_or(0);

        if selected_index > 0 && selected_index <= users.len() {
            let selected_user = &users[selected_index - 1];
            eprintln!(
                "Selected User: {} (Email: {})",
                selected_user.display_name,
                selected_user.get_email()
            );
            break selected_user.clone();
        } else {
            eprintln!("Invalid input. Please try again.");
        }
    };

    eprintln!("Fetching reportees for user ID: {}", selected_user.id);

    println!("id, display_name, mail, job_title, department, office_location, employment_type, location, manager_id, manager_display_name");
    println!("{}, none, none", selected_user);

    fetch_reportee_tree_recursive(&client, &access_token, &selected_user).await?;

    Ok(())
}

const MAX_CONCURRENT_REQUESTS: usize = 10;
static REQUEST_SEMAPHORE: tokio::sync::Semaphore =
    tokio::sync::Semaphore::const_new(MAX_CONCURRENT_REQUESTS);

async fn fetch_users(
    client: &Client,
    access_token: &str,
    url: &str,
) -> anyhow::Result<UsersResponse> {
    let _permit = REQUEST_SEMAPHORE.acquire().await?;
    let mut headers = HeaderMap::new();
    headers.insert(
        AUTHORIZATION,
        format!("Bearer {}", access_token).parse().unwrap(),
    );
    headers.insert(CONTENT_TYPE, "application/json".parse().unwrap());

    // add a sleep here to avoid throttling
    tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;

    let response = client.get(url).headers(headers).send().await?;

    if !response.status().is_success() {
        let status = response.status();
        let response_txt = response.text().await?;
        anyhow::bail!("fetching users; {}: {}", status, response_txt)
    }

    let response_json = response.json().await?;
    Ok(response_json)
}

#[async_recursion]
async fn fetch_reportee_tree_recursive(
    client: &Client,
    access_token: &str,
    manager: &User,
) -> anyhow::Result<()> {
    let mut url = format!(
        "https://graph.microsoft.com/beta/users/{}/directReports",
        manager.id
    );

    loop {
        let response = fetch_users(client, access_token, &url).await?;
        let reportees = response.value;

        if reportees.is_empty() {
            break;
        }

        for reportee in reportees {
            println!("{}, {}, {}", reportee, manager.id, manager.display_name);

            fetch_reportee_tree_recursive(client, access_token, &reportee).await?;
        }

        if let Some(next_link) = response.next_link {
            url = next_link;
        } else {
            break;
        }
    }

    Ok(())
}

fn read_input(prompt: &str) -> io::Result<String> {
    eprint!("{}", prompt);
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input.trim().to_string())
}

impl User {
    fn get_email(&self) -> &str {
        self.mail.as_deref().unwrap_or("unknown")
    }

    fn get_department(&self) -> &str {
        self.department.as_deref().unwrap_or("unknown")
    }

    fn get_job_title(&self) -> &str {
        self.job_title.as_deref().unwrap_or("unknown")
    }

    fn get_office_location(&self) -> &str {
        self.office_location.as_deref().unwrap_or("unknown")
    }

    fn get_category(&self) -> (&str, &str) {
        let unknown = "unknown".to_string();
        let job_title = self.job_title.as_ref().unwrap_or(&unknown);
        let office_location = self.office_location.as_ref().unwrap_or(&unknown);
        let employment_type = if ["CONSULT", "OUTSOURCE", "Outsource"]
            .iter()
            .any(|kw| job_title.contains(kw))
        {
            "Vendor"
        } else {
            "Employee"
        };

        let location = if ["Off-Shore", "Off-Site"]
            .iter()
            .any(|kw| office_location.contains(kw))
        {
            "Off-Shore"
        } else {
            "On-Site"
        };
        (employment_type, location)
    }
}

impl Display for User {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let job_title = self.get_job_title();
        let office_location = self.get_office_location();
        let mail = self.get_email();
        let department = self.get_department();
        let (employment_type, location) = self.get_category();
        write!(
            f,
            "{}, {}, {}, {}, {}, {}, {}, {}",
            self.id,
            self.display_name,
            mail,
            job_title,
            department,
            office_location,
            employment_type,
            location
        )
    }
}

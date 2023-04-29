use reqwest::Client;

use crate::SERVER_URL;

#[derive(Debug, serde::Deserialize, Clone)]
pub struct Task {
    pub id: i64,
    pub name: String,
    #[serde(rename = "percentageDone")]
    pub percentage_done: i32,
}

pub async fn scrape_tasks(lower_limit: u32, upper_limit: u32, client: &Client) -> Vec<Task> {
    
    let mut tasks: Vec<Task> = Vec::new();

    for i in lower_limit..upper_limit {
        let url = format!("{}{}{}", SERVER_URL, "detail/", i);

        let response = client.get(&url).send().await;

        if let Ok(response) = response {
            if response.status().as_u16() != 200 {
                continue;
            }

            let task = response.json::<Task>().await;
            if let Ok(mut task) = task {
                if task.percentage_done > 100 {
                    task.percentage_done = 100;
                } else if task.percentage_done < 0 {
                    task.percentage_done = 0;
                }

                tasks.push(task);
            }
        }

    }
    
    tasks

}
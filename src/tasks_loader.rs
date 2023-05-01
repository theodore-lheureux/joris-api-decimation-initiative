use std::collections::{HashMap, HashSet};

use futures::{stream, StreamExt};
use reqwest::Client;
use select::document::Document;

use crate::{API_URL, SERVER_URL};

const CONCURRENT_REQUESTS: usize = 15;

#[derive(Debug, serde::Deserialize, Clone)]
pub struct Task {
    pub id: i64,
    pub name: String,
    #[serde(rename = "percentageDone")]
    pub percentage_done: i32,
    #[serde(skip)]
    pub username: String,
}

pub async fn count_tasks(client: &Client) -> u32 {
    let res = client.get(format!("{}/index", SERVER_URL)).send().await;

    if let Ok(res) = res {
        if res.status().as_u16() != 200 {
            return 0;
        }
        let html = res.json::<String>().await.unwrap();
        let doc = Document::from(html.as_str());
        let mut html = doc.find(select::predicate::Name("html"));
        let divs = html.next().unwrap().last_child().unwrap().children();

        let mut count = 0;

        for div in divs {
            let tasks = div.children().skip(1).count();
            count += tasks;
        }

        return count as u32;
    }
    0
}

pub async fn find_tasks_username(client: &Client, tasks: &mut Vec<Task>) {
    let res = client.get(format!("{}/index", SERVER_URL)).send().await;
    let mut tasks_map: HashMap<String, &mut Task> = HashMap::new();
    let mut seen_tasks: HashSet<String> = HashSet::new();

    for task in tasks.iter_mut() {
        let task_name = task.name.clone();

        if seen_tasks.contains(&task.name) {
            tasks_map.remove(&task.name);
            continue;
        }
        tasks_map.insert(task.name.clone(), task);
        seen_tasks.insert(task_name);
    }

    if let Ok(res) = res {
        if res.status().as_u16() != 200 {
            return;
        }
        let html = res.json::<String>().await.unwrap();
        let doc = Document::from(html.as_str());
        let mut html = doc.find(select::predicate::Name("html"));
        let divs = html
            .next()
            .unwrap()
            .last_child()
            .unwrap()
            .children()
            .skip(1);

        for div in divs {
            let username = div.first_child().unwrap().text();
            let tasks = div.children().skip(1);

            for task in tasks {
                let name = task.text();
                if let Some(task) = tasks_map.get_mut(&name) {
                    task.username = username.clone();
                }
            }
        }
    }
}

pub async fn scrape_tasks(client: &Client) -> Vec<Task> {
    let mut tasks: Vec<Task> = Vec::new();
    let tasks_count = count_tasks(client).await;
    let mut i = 2;

    while tasks.len() < tasks_count as usize {
        // send 10 requests at a time
        let mut requests = Vec::new();
        for j in i..i + CONCURRENT_REQUESTS {
            requests.push(client.get(format!("{}detail/{}", API_URL, j)));
        }

        let mut responses = stream::iter(requests)
            .map(|req| req.send())
            .buffer_unordered(CONCURRENT_REQUESTS);

        while let Some(res) = responses.next().await {
            if let Ok(res) = res {
                if res.status().as_u16() != 200 {
                    continue;
                }
                let task = res.json::<Task>().await.unwrap();
                tasks.push(task);
            }
        }

        i += CONCURRENT_REQUESTS;

    }

    find_tasks_username(client, &mut tasks).await;
    tasks
}

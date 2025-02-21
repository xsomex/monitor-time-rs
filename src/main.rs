use std::{
    fs::{self, File},
    io::Read,
};

use futures::TryStreamExt;
use mongodb::{
    bson::{doc, Document}, Client, Collection
};
use regex::Regex;

#[tokio::main]
async fn main() -> mongodb::error::Result<()> {
    let mut config: String = String::new();
    File::open("./config").unwrap().read_to_string(&mut config).unwrap();
    let uri = config.lines().collect::<Vec<&str>>()[0];
    let client = Client::with_uri_str(uri).await?;
    let database = client.database("code_time");
    let collection: Collection<Document> = database.collection("time");
    let mut file_content: String = String::from("");

    let file_name = env!("HOME").to_owned() + "/.time";
    File::open(&file_name)
        .unwrap()
        .read_to_string(&mut file_content)
        .unwrap();
    fs::remove_file(&file_name).unwrap();

    let regex = Regex::new(r#"([a-z]*) (\d*) "(.*)""#).unwrap();

    for (_, [f1, f2, f3]) in regex
        .captures_iter(&file_content)
        .map(|caps| caps.extract())
    {
        collection
            .insert_one(doc! {
                "event": f1,
                "timestamp": f2.parse::<i64>().unwrap(),    // ms
                "file": f3,
            })
            .await?;
    }

    let mut cursor = collection
        .find(doc! {"event": doc! {"$exists": true}})
        .sort(doc! {"timestamp": 1})
        .await?;

    let mut enter_event: (i64, String) = (0, String::from(""));
    while let Some(result) = cursor.try_next().await? {
        let timestamp = result.get_i64("timestamp").unwrap();
        let file = result.get_str("file").unwrap().to_string();
        match result.get_str("event").unwrap() {
            "enter" => enter_event = (timestamp, file),
            "leave" => {
                if enter_event.1 == file {
                    collection
                        .insert_one(doc! {
                            "begin": enter_event.0,
                            "duration": timestamp-enter_event.0,
                            "file": file,
                        })
                        .await?;
                    }
            }
            _ => panic!(),
        }
    }

    collection.delete_many(doc! {"event": doc!{"$exists": true}}).await?;

    println!("Total time: {}", millis_to_readable(sum_time(collection).await?));   

    Ok(())
}

async fn sum_time(collection: Collection<Document>) -> mongodb::error::Result<i64> {
    let mut cursor = collection.aggregate([doc! {
        "$group": doc! {
            "_id": "null",
            "total_time": doc! {
                "$sum": "$duration"
            }
        }
    }]).await?;
    while let Some(result) = cursor.try_next().await? {
        return Ok(result.get_i64("total_time").unwrap());
    }
    Ok(0)
}

fn millis_to_readable(ms: i64) -> String {
    if ms < 0 {
        panic!()
    }
    let mut s = ms/1000;
    let mut min = s/60;
    s -= min*60;
    let h = s/60;
    min -= h*60;
    format!("{}h {}min {}s", h, min, s)
}

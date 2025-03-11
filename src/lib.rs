use std::{
    fs::{self, File},
    io::Read,
};

use futures::TryStreamExt;
use mongodb::{
    Collection,
    bson::{Document, doc},
    Client
};
use regex::Regex;

pub async fn try_open_client(uri: &str) -> Result<Collection<Document>, mongodb::error::Result<()>> {
    let client = match Client::with_uri_str(uri).await {
        Err(e) => return Err(Err(e)),
        Ok(v) => v
    };
    let database = client.database("code_time");
    let collection: Collection<Document> = database.collection("time");
    Ok(collection)
}

pub async fn try_add_from_file(path: &str, collection: &Collection<Document>) -> Result<(), ()> {
    let mut file_content: String = String::from("");

    match File::open(path) {
        Ok(mut f) => match f.read_to_string(&mut file_content) {
            Err(_) => return Err(()),
            _ => (),
        },
        Err(_) => return Err(()),
    };

    match fs::remove_file(path) {
        Err(_) => return Err(()),
        _ => (),
    };

    let regex = match Regex::new(r#"([a-z]*) (\d*) "(.*)""#) {
        Ok(r) => r,
        Err(_) => return Err(()),
    };

    for (_, [f1, f2, f3]) in regex
        .captures_iter(&file_content)
        .map(|caps| caps.extract())
    {
        match collection
            .insert_one(doc! {
                "event": f1,
                "timestamp": match f2.parse::<i64>() {
                    Err(_) => return Err(()),
                    Ok(t) => t,
                },    // ms
                "file": f3,
            })
            .await
        {
            Err(_) => return Err(()),
            _ => (),
        };
    }

    let mut cursor = match collection
        .find(doc! {"event": doc! {"$exists": true}})
        .sort(doc! {"timestamp": 1})
        .await
    {
        Ok(v) => v,
        Err(_) => return Err(()),
    };

    let mut enter_event: (i64, String) = (0, String::from(""));

    while let Some(result) = match cursor.try_next().await {
        Ok(v) => v,
        Err(_) => return Err(()),
    } {
        let timestamp = match result.get_i64("timestamp") {
            Ok(v) => v,
            Err(_) => return Err(()),
        };

        let file = match result.get_str("file") {
            Ok(v) => v,
            Err(_) => return Err(()),
        }
        .to_string();

        match match result.get_str("event") {
            Ok(v) => v,
            Err(_) => return Err(()),
        } {
            "enter" => enter_event = (timestamp, file),
            "leave" => {
                if enter_event.1 == file {
                    match collection
                        .insert_one(doc! {
                            "begin": enter_event.0,
                            "duration": timestamp-enter_event.0,
                            "file": file,
                        })
                        .await
                    {
                        Err(_) => return Err(()),
                        _ => (),
                    };
                }
            }
            _ => return Err(()),
        }
    }

    match collection
        .delete_many(doc! {"event": doc!{"$exists": true}})
        .await
    {
        Err(_) => return Err(()),
        _ => (),
    };

    Ok(())
}

pub async fn sum_time(collection: Collection<Document>) -> mongodb::error::Result<i64> {
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

pub fn millis_to_readable(ms: i64) -> String {
    if ms < 0 {
        panic!()
    }
    let mut s = ms/1000;
    let mut min = s/60;
    s -= min*60;
    let h = min/60;
    min -= h*60;
    format!("{}h {}min {}s", h, min, s)
}

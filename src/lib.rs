use futures::TryStreamExt;
use mongodb::{
    Client, Collection,
    bson::{Document, doc},
};
use std::process::Command;

pub mod parse;
pub mod display;

pub async fn try_open_client(
    uri: &str,
) -> Result<Collection<Document>, mongodb::error::Result<()>> {
    let client = match Client::with_uri_str(uri).await {
        Err(e) => return Err(Err(e)),
        Ok(v) => v,
    };
    let database = client.database("code_time");
    let collection: Collection<Document> = database.collection("time");
    Ok(collection)
}

pub async fn sum_time(collection: &Collection<Document>) -> mongodb::error::Result<i64> {
    let mut cursor = collection
        .aggregate([doc! {
            "$group": doc! {
                "_id": "null",
                "total_time": doc! {
                    "$sum": "$duration"
                }
            }
        }])
        .await?;
    while let Some(result) = cursor.try_next().await? {
        return Ok(result.get_i64("total_time").unwrap());
    }
    Ok(0)
}

pub async fn latest(collection: &Collection<Document>, s: i64) -> mongodb::error::Result<i64> {
    let from = String::from_utf8(Command::new("date").arg("+%s").output().unwrap().stdout).unwrap();
    let from = (from[0..from.len() - 1].parse::<i64>().unwrap() - s) * 1000;
    let mut cursor = collection
        .aggregate([
            doc! {
                "$match": doc! {
                    "begin": doc! {
                        "$gte": from
                    }
                }
            },
            doc! {
                "$group": doc! {
                    "_id": "null",
                    "total_time": doc! {
                        "$sum": "$duration"
                    }
                }
            },
        ])
        .await?;
    while let Some(result) = cursor.try_next().await? {
        return Ok(result.get_i64("total_time").unwrap());
    }
    Ok(0)
}

pub async fn daily_average(collection: &Collection<Document>) -> mongodb::error::Result<f64> {
    let mut cursor = collection
        .aggregate([
            doc! {
                "$project": {
                    "day": {
                        "$dateToString": {
                            "format": "%Y-%m-%d",
                            "date": { "$toDate": { "$multiply": ["$begin", 1] } }
                        }
                    },
                    "duration": 1
                }
            },
            doc! {
                "$group": doc! {
                    "_id": "$day",
                    "total_time": doc! {
                        "$sum": "$duration"
                    }
                }
            },
            doc! {
                "$group": doc! {
                    "_id": "null",
                    "average": doc! { "$avg": "$total_time" }
                }
            },
        ])
        .await?;
    while let Some(result) = cursor.try_next().await? {
        return Ok(result.get_f64("average").unwrap());
    }
    Ok(0.0)
}

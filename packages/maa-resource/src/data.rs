use serde::Deserialize;
use time::{format_description::BorrowedFormatItem, OffsetDateTime, PrimitiveDateTime};

const DATE_TIME_FORMAT: &[BorrowedFormatItem<'_>] =
    time::macros::format_description!("[year]-[month]-[day] [hour]:[minute]:[second].[subsecond]");

time::serde::format_description!(custom_date_format, PrimitiveDateTime, DATE_TIME_FORMAT);

/// 对应 `resource/version.json`
#[derive(Debug, Deserialize)]
pub struct Version {
    pub activity: Activity,
    pub gacha: Gacha,

    // maa 这里的时间格式并不标准（）
    #[serde(with = "custom_date_format")]
    pub last_updated: PrimitiveDateTime,
}

impl PartialEq for Version {
    fn eq(&self, other: &Self) -> bool {
        self.last_updated == other.last_updated
    }
}

#[derive(Debug, Deserialize)]
pub struct Activity {
    pub name: String,
    #[serde(with = "time::serde::timestamp")]
    pub time: OffsetDateTime,
}

#[derive(Debug, Deserialize)]
pub struct Gacha {
    pub pool: String,
    #[serde(with = "time::serde::timestamp")]
    pub time: OffsetDateTime,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_deserialize() {
        let json = r#"{
    "activity": {
        "name": "泰拉饭",
        "time": 1725249600
    },
    "gacha": {
        "pool": "泰拉饭，呜呼，泰拉饭",
        "time": 1725249600
    },
    "last_updated": "2024-09-15 20:04:19.394"
}"#;

        println!("{}", json);
        let version: Version = serde_json::from_str(json).unwrap();
        println!("{:?}", version);
    }
}

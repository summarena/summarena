use rusqlite::Connection;

use crate::defs::InputItem;
use crate::defs::LiveSourceSpec;

fn get_db_connection() -> Connection {
    Connection::open("omni.db").unwrap()
}

pub fn ingest(source: &LiveSourceSpec, input_item: InputItem) {
    let conn = get_db_connection();
    conn.execute(
        "
            INSERT INTO input_items
                (uri, live_source_uri, text, vision)
            VALUES
                (?1, ?2, ?3, ?4)
        ",
        (
            &input_item.uri,
            &input_item.live_source_uri,
            &input_item.text,
            &input_item.vision.as_deref(),
        ),
    ).unwrap();
}

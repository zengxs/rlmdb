fn main() -> Result<(), Box<dyn std::error::Error>> {
    write_data()?;
    read_data()?;

    Ok(())
}

fn write_data() -> Result<(), Box<dyn std::error::Error>> {
    let env = rlmdb::DBEnvBuilder::new("test.mdb")
        .set_map_size(1 * 1024 * 1024 * 1024) // 1GB
        .set_max_readers(10)
        .set_max_dbs(5)
        .open(None)?;

    let txn = env.begin_txn()?;
    let db = env.open_db::<&str, Vec<u8>>(&txn, None)?;

    txn.put(&db, "key1", "value1".into(), None)?;
    println!("Inserted key1 with value1");
    txn.put(&db, "key2", "value2".into(), None)?;
    println!("Inserted key2 with value2");

    txn.commit()?;
    println!("Data written successfully");

    Ok(())
}

fn read_data() -> Result<(), Box<dyn std::error::Error>> {
    let env = rlmdb::DBEnvBuilder::new("test.mdb")
        .set_map_size(10 * 1024 * 1024) // 1GB
        .set_max_readers(10)
        .set_max_dbs(5)
        .open(None)?;

    let txn = env.begin_txn_read_only()?;
    let db = env.open_db::<&str, Vec<u8>>(&txn, None)?;

    if let Some(value) = txn.get(&db, "key1")? {
        println!(
            "Retrieved value for 'key1': {}",
            String::from_utf8(value).unwrap()
        );
    } else {
        println!("No value found for 'key1'");
    }

    Ok(())
}

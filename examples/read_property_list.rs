// cargo run --example read_property_list -- --addr "192.168.1.249:47808" --device-id 79079
// cargo run --example read_property_list --no-default-features -- --addr "192.168.1.249:47808" --device-id 79079

use clap::Parser;
use common::MySocket;
use embedded_bacnet::{
    application_protocol::{
        primitives::data_value::ApplicationDataValue,
        services::read_property::ReadProperty,
    },
    common::{
        object_id::{ObjectId, ObjectType},
        property_id::PropertyId,
    },
    simple::BacnetError,
};

mod common;

/// A Bacnet Client example to read the list of properties for the device
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// IP address with port e.g. "192.168.1.249:47808"
    #[arg(short, long)]
    addr: String,

    /// Device ID of the controller e.g. 79079
    #[arg(short, long)]
    device_id: u32,
}

#[tokio::main]
async fn main() -> Result<(), BacnetError<MySocket>> {
    // setup
    let args = Args::parse();
    let bacnet = common::get_bacnet_socket(&args.addr).await?;
    let mut buf = vec![0; 1500];

    // fetch
    let object_id = ObjectId::new(ObjectType::ObjectDevice, args.device_id);
    let request = ReadProperty::new(object_id, PropertyId::PropObjectList);
    let result = bacnet.read_property(&mut buf, request).await?;

    // print
    for item in result.property_value {
        match item {
            Ok(ApplicationDataValue::ObjectId(object_id)) => {
                println!("{:?}", object_id);
            }
            Ok(other) => {
                println!("Unexpected: {:?}", other);
            }
            Err(e) => {
                println!("Error: {:?}", e);
            }
        }
    }

    Ok(())
}

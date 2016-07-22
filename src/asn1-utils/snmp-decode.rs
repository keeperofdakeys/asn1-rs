extern crate asn1_cereal;
extern crate argparse;
extern crate serde;
extern crate serde_json;

use asn1_cereal::{tag, byte};
use asn1_cereal::ber::stream;

// SNMP ASN.1 Definition
// https://tools.ietf.org/html/rfc1157#page-30

type ObjectIdentifier = u64;
type NetworkAddress = u64;
type ObjectName = String;

struct Message {
  version: i32,
  community: String,
  data: PDUs,
}

enum PDUs {
  get_request(GetRequest),
  get_next_request(GetNextRequest),
  get_response(GetResponse),
  set_request(SetRequest),
  trap(TrapPDU),
}

struct GetRequest(PDU);

struct GetNextRequest(PDU);

struct GetResponse(PDU);

struct SetRequest(PDU);

struct PDU {
  request_id: i32,
  error_status: i32,
  error_index: i32,
  variable_bindings: VarBindList,
}

struct TrapPDU {
  enterprise: ObjectIdentifier,
  agent_addr: NetworkAddress,
  generic_trap: i32,
  specific_trap: i32,
  time_stamp: TimeTicks,
  variable_bindings: VarBindList,
}

struct VarBind {
  name: ObjectName,
  value: ObjectSyntax,
}

type VarBindList = Vec<VarBind>;

use std::io;
use std::io::Read;
use std::fs;
use std::path::Path;
use std::collections::BTreeMap;
use argparse::{ArgumentParser, StoreTrue, StoreOption};
use serde_json::value::Value;
use serde_json::ser::to_string_pretty;

fn main() {
  let opts = parse_args();

  let path = Path::new(opts.file.as_ref().unwrap());
  if !path.is_file() {
    panic!("Supplied file does not exist");
  }

  // Create a buffered reader from the file.
  let reader = io::BufReader::new(fs::File::open(path).unwrap()).bytes();
}

struct ProgOpts {
  file: Option<String>,
  verbose: bool,
}

fn parse_args() -> ProgOpts {
  let mut opts = ProgOpts {
    file: None,
    verbose: false,
  };

  {
    let mut ap = ArgumentParser::new();
    ap.set_description("Decode ASN.1 files");
    ap.refer(&mut opts.verbose)
      .add_option(&["-v", "--verbose"], StoreTrue, "Verbose output");
    ap.refer(&mut opts.file)
      .add_argument("file", StoreOption, "ASN.1 file to decode");
    ap.parse_args_or_exit();
  }
  opts
}

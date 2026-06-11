use crate::config_feature::execute_get::get_from_cdb;
use crate::CONFIG_DB;
use core::time;
use std::io::Sink;
use std::io::{BufRead, Read};
// use std::fs::File;
use crate::database;
use anyhow::Context;
use chrono::{DateTime, TimeDelta, Utc};
use std::path::Path;
use std::path::PathBuf;
use winnow::ascii::alphanumeric0;
use winnow::error::ContextError;
use winnow::prelude::*;
use winnow::stream::ToUsize;
use winnow::stream::{AsBStr, AsBytes};
use winnow::Result;

pub async fn parse(db: std::sync::Arc<database::Database>) -> Result<()> {
    let dir = get_from_cdb(&"dir".to_string()).await;
    let dbfilename = get_from_cdb(&"dbfilename".to_string()).await;
    let mut parsed_single_db_vec: Vec<(Option<String>, Option<String>, Option<String>)> =
        Vec::new();
    let path = Path::new(&dir).join(dbfilename);
    let file = std::fs::File::open(path);
    match file {
        Ok(f) => {
            let mut reader = std::io::BufReader::new(f);

            let mut buf: Vec<u8> = Vec::new();
            match reader.read_to_end(&mut buf) {
                Ok(_data) => {
                    // eprintln!("DATA printing press");
                    // eprintln!("{:?}", buf);
                    let mut rdb_stream = winnow::stream::Bytes::new(&buf);

                    //Parse away the header section of RDB file
                    let rdb_header = parse_char.parse_next(&mut rdb_stream);

                    //repeat auxiliary parser until no auxiliary data is left.
                    let parsed_metadata: winnow::Result<Vec<Option<(String, String)>>, _> =
                        winnow::combinator::repeat(
                            0..,
                            winnow::combinator::cond(
                                check_if_byte_fa(&mut rdb_stream),
                                parse_metadata,
                            ),
                        )
                        .parse_next(&mut rdb_stream);

                    // eprintln!("{:?}", parsed_metadata.unwrap());
                    // eprintln!("{:?}", rdb_stream);

                    let parsed_database = parse_database_section.parse_next(&mut rdb_stream);
                    // eprintln!("AFTER READING DATABASE SECTION: {:?}", rdb_stream);
                    parsed_single_db_vec = parsed_database.unwrap();
                    // eprintln!("{}", rdb_stream);
                }
                Err(e) => {
                    eprintln!("Error while reading .rdb file\n Error: {}", e);
                }
            }
        }
        Err(e) => {
            eprintln!("Error occured during File Read\n{}", e);
        }
    }

    // after parsing the .rdb file, put everything in the redis hashmap.
    // this goes against the principle of cohesion and whatever, but I had
    // no better idea at the time.

    for item in parsed_single_db_vec {
        let (key_opt, val_opt, ts) = item;
        let key = key_opt.unwrap();
        let value = val_opt.unwrap();

        if let Some(time) = ts {
            let millisecond_expire_ms_option = time.parse::<i64>();
            let _ = db
                .add_to_db_with_expire_ms(
                    key,
                    value,
                    TimeDelta::try_milliseconds(
                        millisecond_expire_ms_option.unwrap() - Utc::now().timestamp_millis(),
                    )
                    .unwrap(),
                )
                .await;
        } else {
            let _ = db.add_to_db_without_expire_ms(key, value).await;
        }
    }

    Ok(())
}
fn check_if_byte_fa(input: &mut &winnow::stream::Bytes) -> bool {
    let possible_fa: winnow::Result<&[u8], winnow::error::ContextError> =
        winnow::combinator::peek(winnow::token::take(1usize)).parse_next(input);

    let pfa = possible_fa.unwrap();

    let value = format!("{:X}", pfa.first().unwrap());
    value == "FA".to_string()
}

fn parse_char(input: &mut &winnow::stream::Bytes) -> Result<String> {
    let header_byte_slice_wrapped = winnow::token::take(9usize).parse_next(input);
    match header_byte_slice_wrapped {
        Ok(hbs) => {
            let ans = hbs.iter().map(|c| *c as char).collect::<String>();
            Ok(ans)
        }
        Err(e) => Err(e),
    }
}

fn parse_char_nth(input: &mut &winnow::stream::Bytes, token_count: usize) -> String {
    let header_byte_slice_wrapped: Result<&[u8], winnow::error::ContextError> =
        winnow::token::take(token_count).parse_next(input);
    match header_byte_slice_wrapped {
        Ok(hbs) => {
            let redis_header = winnow::Bytes::new(hbs).to_ascii_lowercase();
            // eprintln!("{:?}", String::from_utf8(redis_header.clone()).unwrap());
            // Ok(String::from_utf8(redis_header).unwrap())

            String::from_utf8(redis_header).unwrap()
        }
        Err(_e) => "Huge blunder".to_string(),
    }
}

fn peek_at_one_token_now(input: &mut &winnow::stream::Bytes) -> Result<u8> {
    let single_token: Result<&[u8], winnow::error::ContextError> =
        winnow::combinator::peek(winnow::token::take(1usize)).parse_next(input);
    match single_token {
        Ok(token_arr) => {
            let fval = token_arr.first();
            match fval {
                Some(val) => Ok(*val),
                _ => Err(winnow::error::ContextError::from_input(input)),
            }
        }
        Err(e) => Err(e),
    }
}

fn u8_slice_to_string(slice: winnow::Result<&[u8], winnow::error::ContextError>) -> String {
    match slice {
        Ok(val) => {
            let ans = val.iter().map(|c| *c as char).collect::<String>();
            ans
        }
        Err(e) => e.to_string(),
    }
}

fn u8_slice_to_usize(slice: winnow::Result<&[u8], winnow::error::ContextError>) -> Option<usize> {
    match slice {
        Ok(val) => {
            if !val.is_empty() {
                Some(*val.first().unwrap() as usize)
            } else {
                None
            }
        }
        Err(e) => None,
    }
}

pub fn get_aux_size(input: &mut &winnow::stream::Bytes) -> Result<usize> {
    let enc_byte: winnow::Result<&[u8], winnow::error::ContextError> =
        winnow::combinator::peek(winnow::token::take(1usize)).parse_next(input);
    let mut final_value = 0_usize;
    if let Ok(val) = enc_byte {
        let mut value = val.first().unwrap().clone();
        value = value >> 6;

        match value {
            0 => {
                //consume 1 byte and return it.
                let single_byte: winnow::Result<u8, winnow::error::ContextError> =
                    winnow::binary::be_u8.parse_next(input);
                final_value = single_byte.unwrap().to_usize();
            }
            1 => {
                let rstring: winnow::Result<u16, winnow::error::ContextError> =
                    winnow::binary::be_u16.parse_next(input);
                let opr: u16 = u16::from_str_radix("0011111111111111", 2).unwrap();
                let sz_16b = (rstring.unwrap() & opr).to_usize();
                final_value = sz_16b;
            }
            2 => {
                let _: winnow::Result<&[u8], winnow::error::ContextError> =
                    winnow::token::take(1usize).parse_next(input);
                // let four_bytes:winnow::Result<&[u8], winnow::error::ContextError> = winnow::token::take(4usize).parse_next(input);
                let four_byte_val: winnow::Result<u32, winnow::error::ContextError> =
                    winnow::binary::be_u32.parse_next(input);

                final_value = four_byte_val.unwrap().to_usize();
            }
            _ => {
                /*This must be handled in future, when we encounter such situation*/
                // eprintln!("HEY BUDDY HANDLE THIS AS WELL = {:08b}", value);
                // eprintln!("ARE YOU STUNNED NOW\n{:?}", input);
                let opr = u8::from_str_radix("00111111", 2).unwrap();
                let if_c0: winnow::Result<u8, winnow::error::ContextError> =
                    winnow::binary::be_u8.parse_next(input);
                let if_c0: u8 = if_c0.unwrap() & opr;

                // eprintln!("ARE YOU STUNNED NOW\n{:?}", input);
                if if_c0 == 0 {
                    final_value = 1_usize;

                    // eprintln!(
                    //     "Final_value = {}\nARE YOU STUNNED NOW\n{:?}",
                    //     final_value, input
                    // );
                } else if if_c0 == 1 {
                    final_value = 2_usize
                } else if if_c0 == 2 {
                    final_value = 4_usize;
                } else {
                    final_value = 0_usize;
                }
            }
        }
    }
    Ok(final_value)
}
// fn get_aux_initiate_byte(input: &mut &winnow::stream::Bytes) -> winnow::Result<String> {}
fn parse_metadata_with_integer_value(
    aux_val_val: Result<&[u8], winnow::error::ContextError>,
    aux_sz: winnow::Result<usize, winnow::error::ContextError>,
) -> String {
    let mut val2 = String::new();
    let mut str_val2 = aux_val_val.unwrap();
    match aux_sz.unwrap() {
        1_usize => {
            let mut aux_val_in_byte = winnow::Bytes::new(str_val2);
            let intermediate_value: winnow::Result<u8, winnow::error::ContextError> =
                winnow::binary::be_u8.parse_next(&mut aux_val_in_byte);
            val2 = intermediate_value.unwrap().to_string();
        }
        2_usize => {
            let mut aux_val_in_byte = winnow::Bytes::new(str_val2);
            let intermediate_value: winnow::Result<u16, winnow::error::ContextError> =
                winnow::binary::be_u16.parse_next(&mut aux_val_in_byte);
            val2 = intermediate_value.unwrap().to_string();
        }
        4_usize => {
            let mut aux_val_in_byte = winnow::Bytes::new(str_val2);
            let intermediate_value: winnow::Result<u32, winnow::error::ContextError> =
                winnow::binary::be_u32.parse_next(&mut aux_val_in_byte);
            val2 = intermediate_value.unwrap().to_string();
        }
        _ => {
            val2 = 0_usize.to_string();
        }
    }
    val2
}
fn parse_metadata(input: &mut &winnow::stream::Bytes) -> winnow::Result<(String, String)> {
    let mdata_begin: Result<&[u8], winnow::error::ContextError> =
        winnow::combinator::peek(winnow::token::take(1usize)).parse_next(input);

    // eprintln!("I am stunnned parse metadata:\n{}", input);
    let asbyte = mdata_begin.unwrap();
    let value = format!("{:X}", asbyte.first().unwrap());
    match value.as_str() {
        "FA" => {
            //consume the byte that represents FA.
            let _: winnow::Result<&[u8], winnow::error::ContextError> =
                winnow::token::take(1usize).parse_next(input);

            let aux_sz = get_aux_size.parse_next(input);
            let aux_val_name: winnow::Result<&[u8], winnow::error::ContextError> =
                winnow::token::take(aux_sz.unwrap()).parse_next(input);
            let aux_sz = get_aux_size.parse_next(input);
            // eprintln!("VAL2 SIZE = {:?}", aux_sz);
            let aux_val_val: winnow::Result<&[u8], winnow::error::ContextError> =
                winnow::token::take(aux_sz.clone().unwrap()).parse_next(input);

            let val1 = u8_slice_to_string(aux_val_name);
            let mut val2 = String::new();

            if val1.as_str() == "redis-bits"
                || val1.as_str() == "ctime"
                || val1.as_str() == "used-mem"
                || val1.as_str() == "aof-base"
            {
                val2 = parse_metadata_with_integer_value(aux_val_val, aux_sz);
            } else {
                val2 = u8_slice_to_string(aux_val_val);
            }
            Ok((val1, val2))
        }
        _ => Err(winnow::error::ContextError::new()),
        // _ => Err("STOP"),
    }
}
fn get_db_section_len_encoding(input: &mut &winnow::stream::Bytes) -> Result<(bool, usize)> {
    let enc_byte: winnow::Result<&[u8], winnow::error::ContextError> =
        winnow::combinator::peek(winnow::token::take(1usize)).parse_next(input);
    let mut final_value = 0_usize;
    let mut int_or_not = false;
    if let Ok(val) = enc_byte {
        let mut value = val.first().unwrap().clone();
        value = value >> 6;

        match value {
            0 => {
                //consume 1 byte and return it.
                let single_byte: winnow::Result<u8, winnow::error::ContextError> =
                    winnow::binary::be_u8.parse_next(input);
                final_value = single_byte.unwrap().to_usize();
            }
            1 => {
                let rstring: winnow::Result<u16, winnow::error::ContextError> =
                    winnow::binary::be_u16.parse_next(input);
                let opr: u16 = u16::from_str_radix("0011111111111111", 2).unwrap();
                let sz_16b = (rstring.unwrap() & opr).to_usize();
                final_value = sz_16b;
            }
            2 => {
                let _: winnow::Result<&[u8], winnow::error::ContextError> =
                    winnow::token::take(1usize).parse_next(input);
                // let four_bytes:winnow::Result<&[u8], winnow::error::ContextError> = winnow::token::take(4usize).parse_next(input);
                let four_byte_val: winnow::Result<u32, winnow::error::ContextError> =
                    winnow::binary::be_u32.parse_next(input);

                final_value = four_byte_val.unwrap().to_usize();
            }
            _ => {
                /*This must be handled in future, when we encounter such situation*/
                // eprintln!("HEY BUDDY HANDLE THIS AS WELL = {:08b}", value);
                // eprintln!("ARE YOU STUNNED NOW\n{:?}", input);
                int_or_not = true;
                let opr = u8::from_str_radix("00111111", 2).unwrap();
                let if_c0: winnow::Result<u8, winnow::error::ContextError> =
                    winnow::binary::be_u8.parse_next(input);
                let if_c0: u8 = if_c0.unwrap() & opr;

                if if_c0 == 0 {
                    final_value = 1_usize;
                } else if if_c0 == 1 {
                    final_value = 2_usize
                } else if if_c0 == 2 {
                    final_value = 4_usize;
                } else {
                    final_value = 0_usize;
                }
            }
        }
    }
    Ok((int_or_not, final_value))
}

//returns the actual value instead of returning the bytes to read
//from the stream. That way all logic can be held in one funciton.
fn db_section_length_encoding_decode(
    input: &mut &winnow::Bytes,
) -> Result<usize, winnow::error::ContextError> {
    let peeked_value: Result<u8, winnow::error::ContextError> =
        winnow::combinator::peek(winnow::binary::be_u8).parse_next(input);
    let first_two_bits = (peeked_value.unwrap() >> 6) as usize;
    let mut final_return_value = 0_usize;
    match first_two_bits {
        0_usize => {
            let fin_val: winnow::Result<u8, winnow::error::ContextError> =
                winnow::binary::be_u8.parse_next(input);

            final_return_value = fin_val.unwrap().to_usize();
        }
        1_usize => {
            let pen_fin_val: winnow::Result<u16, winnow::error::ContextError> =
                winnow::binary::be_u16.parse_next(input);
            let opr = u16::from_str_radix("0011111111111111", 2).unwrap();
            final_return_value = (pen_fin_val.unwrap() & opr).to_usize();
        }
        2_usize => {
            // get rid of the size indicator byte here.
            let _: winnow::Result<&[u8], winnow::error::ContextError> =
                winnow::token::take(1_usize).parse_next(input);
            let pen_fin_val: winnow::Result<u32, winnow::error::ContextError> =
                winnow::binary::be_u32.parse_next(input);
            final_return_value = pen_fin_val.unwrap().to_usize();
        }
        _ => {
            // This case represents the case where the first two bits of the
            // indicator byte equate to 3.

            //consume the byte because now this byte is just a size indicator.
            let size_byte: winnow::Result<u8, winnow::error::ContextError> =
                winnow::binary::be_u8.parse_next(input);
            let opr = u8::from_str_radix("00111111", 2).unwrap();
            let compare_num = (size_byte.unwrap() & opr).to_usize();

            match compare_num {
                0_usize => {
                    let pen_fin_val: winnow::Result<u8, winnow::error::ContextError> =
                        winnow::binary::be_u8.parse_next(input);

                    final_return_value = pen_fin_val.unwrap().to_usize();
                }
                1_usize => {
                    let pen_fin_val: winnow::Result<u16, winnow::error::ContextError> =
                        winnow::binary::be_u16.parse_next(input);
                    final_return_value = pen_fin_val.unwrap().to_usize();
                }
                2_usize => {
                    let pen_fin_val: winnow::Result<u32, winnow::error::ContextError> =
                        winnow::binary::be_u32.parse_next(input);
                    final_return_value = pen_fin_val.unwrap().to_usize();
                }
                _ => {
                    final_return_value = 0_usize;
                }
            }
        }
    }
    Ok(final_return_value)
}

fn db_data_parser(
    input: &mut &winnow::Bytes,
) -> winnow::Result<(Option<String>, Option<String>, Option<String>)> {
    let if_ts_or_not: winnow::Result<u8, winnow::error::ContextError> =
        winnow::combinator::peek(winnow::binary::be_u8).parse_next(input);
    let if_ts_or_not_str = format!("{:X}", if_ts_or_not.unwrap());
    let mut time_stamp: Option<String> = None;
    let mut key: Option<String> = None;
    let mut value: Option<String> = None;

    match if_ts_or_not_str.as_str() {
        "FC" => {
            // consume byte that represents FC
            // represents time in milliseconds
            let _: winnow::Result<&[u8], winnow::error::ContextError> =
                winnow::token::take(1_usize).parse_next(input);
            let ts_in_ms: winnow::Result<u64, winnow::error::ContextError> =
                winnow::binary::le_u64.parse_next(input);
            time_stamp = Some(ts_in_ms.unwrap().to_string());
            // eprintln!("{:?}", time_stamp);
        }
        "FD" => {
            // consume byte that represents FD
            // represents time in seconds, so convert to milliseconds before
            // sending the response.
            let _: winnow::Result<&[u8], winnow::error::ContextError> =
                winnow::token::take(1_usize).parse_next(input);
            let ts_in_seconds: winnow::Result<u32, winnow::error::ContextError> =
                winnow::binary::le_u32.parse_next(input);
            time_stamp = Some((ts_in_seconds.unwrap() * 1000).to_string());
        }
        _ => {
            time_stamp = None;
        }
    }

    let val_type_and_enc: winnow::Result<u8, winnow::error::ContextError> =
        winnow::binary::be_u8.parse_next(input);

    let key_size: winnow::Result<usize, winnow::error::ContextError> =
        db_section_length_encoding_decode.parse_next(input);
    let key_impure: winnow::Result<&[u8], winnow::error::ContextError> =
        winnow::token::take(key_size.unwrap()).parse_next(input);
    key = Some(u8_slice_to_string(key_impure));

    /*
     * currently we are using if statements, but in future we may have to,
     * convert this to a match block or an if-else ladder. Because there
     * are 14 different types defined for Value of redis.
     */

    // 0 means string encoding
    if val_type_and_enc.unwrap() == 0 {
        let value_size = db_section_length_encoding_decode(input);
        let val_unwrap = value_size.unwrap();
        let value_u8: winnow::Result<&[u8], winnow::error::ContextError> =
            winnow::token::take(val_unwrap).parse_next(input);
        let value_str = u8_slice_to_string(value_u8);
        value = Some(value_str);
    }

    Ok((key, value, time_stamp))
}

fn reached_db_end_FF(input: &mut &winnow::Bytes) -> bool {
    let is_ff_u8_wrapped: winnow::Result<u8, winnow::error::ContextError> =
        winnow::combinator::peek(winnow::binary::be_u8).parse_next(input);
    let is_ff_str = format!("{:X}", is_ff_u8_wrapped.unwrap());

    if is_ff_str == "FF" {
        true
    } else {
        false
    }
}

fn parse_database_section(
    input: &mut &winnow::Bytes,
) -> winnow::Result<Vec<(Option<String>, Option<String>, Option<String>)>> {
    let mdata_begin: Result<&[u8], winnow::error::ContextError> =
        winnow::combinator::peek(winnow::token::take(1usize)).parse_next(input);

    let asbyte = mdata_begin.unwrap();
    let fe_byte_value = format!("{:X}", asbyte.first().unwrap());
    // eprintln!("{}", fe_byte_value);
    let mut final_db: Vec<(Option<String>, Option<String>, Option<String>)> = Vec::new();
    if fe_byte_value.as_str() == "FE" {
        //consume FE
        let _: winnow::Result<&[u8], winnow::error::ContextError> =
            winnow::token::take(1_usize).parse_next(input);

        /*

        // figure out to way to recurse over this section below
        // perhaps we can recurse over this section until we encounter
        // FF byte which indicates the beginning of CRC checksum.

        // attempt at recursing the database parsing
        // This attempt will work, but we don't have the need right now
        // handler the case of multiple database in the .rdb file later.
        // For now lets focus on sinle db parsing which works perfectly.
        // let _: winnow::Result<(), winnow::error::ContextError> = winnow::combinator::repeat(
        //     0..,
        //     winnow::combinator::cond(reached_db_end_FF(input), start_parsing_after_the_db_byte),
        // )
        // .parse_next(input);


        */

        //consume the start of DB byte
        let db_index: winnow::Result<usize, winnow::error::ContextError> =
            db_section_length_encoding_decode.parse_next(input);

        /* process hash table from this point forward */

        // consume the FB byte that indicate the start of hash table
        let _: winnow::Result<u8, winnow::error::ContextError> = winnow::binary::be_u8(input);
        let ht_size = db_section_length_encoding_decode.parse_next(input);
        let pairs_with_ts = db_section_length_encoding_decode.parse_next(input);
        // let db_data_in_vec: winnow::Result<
        //     Vec<(Option<String>, Option<String>, Option<String>), winnow::error::ContextError>,
        // > = winnow::combinator::repeat(0..ht_sz_i32, db_data_parser).parse_next(input);

        let db_data_in_vec: winnow::Result<
            Vec<(Option<String>, Option<String>, Option<String>)>,
            winnow::error::ContextError,
        > = winnow::combinator::repeat(ht_size.unwrap(), db_data_parser).parse_next(input);

        final_db = db_data_in_vec.unwrap();
        // eprintln!("TESTING VECTOR ACCUMULATOR: {:?}", db_data_in_vec);
    }

    Ok(final_db)
}

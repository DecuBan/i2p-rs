use nom::{
	branch::alt,
	bytes::complete::{tag, take_till},
	character::complete::{alphanumeric1, multispace1},
	multi::separated_list1,
	sequence::{delimited, tuple},
	IResult, Parser,
};

fn is_space(chr: char) -> bool {
	chr == ' ' || chr == '\t'
}

fn is_next_line(chr: char) -> bool {
	chr == '\n'
}

fn is_space_or_next_line(chr: char) -> bool {
	is_space(chr) || is_next_line(chr)
}

fn is_double_quote(chr: char) -> bool {
	chr == '\"'
}

fn quoted_value(input: &str) -> IResult<&str, &str> {
	delimited(tag("\""), take_till(|c| is_double_quote(c)), tag("\"")).parse(input)
}

fn value(input: &str) -> IResult<&str, &str> {
	take_till(|c| is_space_or_next_line(c)).parse(input)
}

fn key_value(input: &str) -> IResult<&str, (&str, &str)> {
	match tuple((alphanumeric1, tag("="), alt((quoted_value, value)))).parse(input) {
		Ok((next, (k, _, v))) => Ok((next, (k, v))),
		Err(e) => Err(e),
	}
}

fn keys_and_values(input: &str) -> IResult<&str, Vec<(&str, &str)>> {
	separated_list1(multispace1, key_value).parse(input)
}

pub fn sam_hello(input: &str) -> IResult<&str, Vec<(&str, &str)>> {
	delimited(tag("HELLO REPLY "), keys_and_values, tag("\n")).parse(input)
}

pub fn sam_session_status(input: &str) -> IResult<&str, Vec<(&str, &str)>> {
	delimited(tag("SESSION STATUS "), keys_and_values, tag("\n")).parse(input)
}

pub fn sam_stream_status(input: &str) -> IResult<&str, Vec<(&str, &str)>> {
	delimited(tag("STREAM STATUS "), keys_and_values, tag("\n")).parse(input)
}

pub fn sam_naming_reply(input: &str) -> IResult<&str, Vec<(&str, &str)>> {
	delimited(tag("NAMING REPLY "), keys_and_values, tag("\n")).parse(input)
}

pub fn sam_dest_reply(input: &str) -> IResult<&str, Vec<(&str, &str)>> {
	delimited(tag("DEST REPLY "), keys_and_values, tag("\n")).parse(input)
}

#[cfg(test)]
mod tests {
	use nom::{error::ErrorKind, Err};

	fn err_kind<I>(err: Err<nom::error::Error<I>>) -> ErrorKind {
		match err {
			Err::Error(e) | Err::Failure(e) => e.code,
			Err::Incomplete(_) => panic!("incomplete input"),
		}
	}

	#[test]
	fn hello() {
		use crate::parsers::sam_hello;

		assert_eq!(
			sam_hello("HELLO REPLY RESULT=OK VERSION=3.1\n"),
			Ok(("", vec![("RESULT", "OK"), ("VERSION", "3.1")]))
		);
		assert_eq!(
			sam_hello("HELLO REPLY RESULT=NOVERSION\n"),
			Ok(("", vec![("RESULT", "NOVERSION")]))
		);
		assert_eq!(
			sam_hello("HELLO REPLY RESULT=I2P_ERROR MESSAGE=\"Something failed\"\n"),
			Ok((
				"",
				vec![("RESULT", "I2P_ERROR"), ("MESSAGE", "Something failed")]
			))
		);
	}

	#[test]
	fn session_status() {
		use crate::parsers::sam_session_status;

		assert_eq!(
			sam_session_status("SESSION STATUS RESULT=OK DESTINATION=privkey\n"),
			Ok(("", vec![("RESULT", "OK"), ("DESTINATION", "privkey")]))
		);
		assert_eq!(
			sam_session_status("SESSION STATUS RESULT=DUPLICATED_ID\n"),
			Ok(("", vec![("RESULT", "DUPLICATED_ID")]))
		);
	}

	#[test]
	fn stream_status() {
		use crate::parsers::sam_stream_status;

		assert_eq!(
			sam_stream_status("STREAM STATUS RESULT=OK\n"),
			Ok(("", vec![("RESULT", "OK")]))
		);
		assert_eq!(
			sam_stream_status(
				"STREAM STATUS RESULT=CANT_REACH_PEER MESSAGE=\"Can't reach peer\"\n"
			),
			Ok((
				"",
				vec![
					("RESULT", "CANT_REACH_PEER"),
					("MESSAGE", "Can't reach peer")
				]
			))
		);
	}

	#[test]
	fn naming_reply() {
		use crate::parsers::sam_naming_reply;

		assert_eq!(
			sam_naming_reply("NAMING REPLY RESULT=OK NAME=name VALUE=dest\n"),
			Ok((
				"",
				vec![("RESULT", "OK"), ("NAME", "name"), ("VALUE", "dest")]
			))
		);
		assert_eq!(
			sam_naming_reply("NAMING REPLY RESULT=KEY_NOT_FOUND\n"),
			Ok(("", vec![("RESULT", "KEY_NOT_FOUND")]))
		);

		assert_eq!(
			err_kind(sam_naming_reply("NAMINGREPLY RESULT=KEY_NOT_FOUND\n").unwrap_err()),
			ErrorKind::Tag
		);

		assert_eq!(
			err_kind(sam_naming_reply("NAMING  REPLY RESULT=KEY_NOT_FOUND\n").unwrap_err()),
			ErrorKind::Tag
		);
	}

	#[test]
	fn dest_reply() {
		use crate::parsers::sam_dest_reply;

		assert_eq!(
			sam_dest_reply("DEST REPLY PUB=foo PRIV=foobar\n"),
			Ok(("", vec![("PUB", "foo"), ("PRIV", "foobar")]))
		);
	}
}

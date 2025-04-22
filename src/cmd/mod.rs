use std::io::{Cursor, Read};

use crate::frame::Frame;

#[derive(Debug)]
pub enum Method {
    Null,
    Query,
    Insert,
    Update,
    Delete,
}

#[derive(Debug)]
pub struct Cmd {
    method: Method,
    table: String,
    columns: Vec<String>,
    values: Vec<String>,
    expression: Option<String>,
}

impl Cmd {
    pub fn build(method: Method) -> Cmd {
        Cmd {
            method,
            table: String::new(),
            columns: Vec::new(),
            values: Vec::new(),
            expression: None,
        }
    }

    pub fn from_vec(src: &[u8]) -> Cmd {
        let mut cmd = Cmd {
            method: Method::Null,
            table: String::new(),
            columns: Vec::new(),
            values: Vec::new(),
            expression: None,
        };

        let mut src = Cursor::new(src);

        //TODO: handle columns len and values len
        while let Some((tag, data)) = get_line(&mut src) {
            match tag[0] {
                b'&' => cmd.method = Method::Query,
                b'+' => cmd.method = Method::Insert,
                b'*' => cmd.method = Method::Update,
                b'-' => cmd.method = Method::Delete,
                b'#' => cmd.table = String::from_utf8(data.to_vec()).unwrap(),
                b'@' => println!("columns len: {}", data[0]),
                b'^' => cmd.columns.push(String::from_utf8(data.to_vec()).unwrap()),
                b'%' => println!("values len: {}", data[0]),
                b'!' => cmd.values.push(String::from_utf8(data.to_vec()).unwrap()),
                b'?' => cmd.expression = Some(String::from_utf8(data.to_vec()).unwrap()),
                _ => unimplemented!(),
            }
        }

        cmd
    }

    pub fn set_table(&mut self, table: impl ToString) -> &mut Self {
        self.table = table.to_string();

        self
    }

    pub fn set_column(&mut self, columns: Vec<impl ToString>) -> &mut Self {
        self.columns.clear();

        for column in columns {
            self.columns.push(column.to_string());
        }

        self
    }

    pub fn set_values(&mut self, values: Vec<impl ToString>) -> &mut Self {
        self.values.clear();

        for value in values {
            self.values.push(value.to_string())
        }

        self
    }

    pub fn set_condition(&mut self, condition: impl ToString) -> &mut Self {
        self.expression = Some(condition.to_string());

        self
    }

    pub fn into_frame(&mut self) -> Frame {
        let mut frame = Frame::array();

        match self.method {
            Method::Query => {
                frame.push_sign(b'&');
            }
            Method::Insert => {
                frame.push_sign(b'+');
            }
            Method::Update => {
                frame.push_sign(b'*');
            }
            Method::Delete => {
                frame.push_sign(b'-');
            }
            Method::Null => {
                unreachable!()
            }
        }

        frame.push_table(&self.table);
        frame.push_columns(&self.columns);
        frame.push_values(&self.values);
        frame.push_expression(&self.expression);

        frame
    }
}

fn get_line(src: &mut Cursor<&[u8]>) -> Option<([u8; 1], Vec<u8>)> {
    let mut tag = [0u8; 1];
    if src.read_exact(&mut tag).is_err() {
        return None;
    };

    let start = src.position() as usize;
    let end = src.get_ref().len() - 1;

    for i in start..end {
        if src.get_ref()[i] == b'\r' && src.get_ref()[i + 1] == b'\n' {
            // Extract the data to an owned Vec<u8>
            let data = src.get_ref()[start..i].to_vec();
            // Update the position
            src.set_position((i + 2) as u64);
            return Some((tag, data));
        }
    }

    // Handle the case where there's no \r\n at the end
    if start < src.get_ref().len() {
        let data = src.get_ref()[start..].to_vec();
        src.set_position(src.get_ref().len() as u64);
        return Some((tag, data));
    }

    None
}

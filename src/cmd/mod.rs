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
    pub method: Method,
    table: String,
    columns: Vec<String>,
    columns_len: usize,
    values: Vec<String>,
    values_len: usize,
    expression: Option<String>,
}

impl Cmd {
    pub fn build(method: Method) -> Cmd {
        Cmd {
            method,
            table: String::new(),
            columns: Vec::new(),
            columns_len: 0,
            values: Vec::new(),
            values_len: 0,
            expression: None,
        }
    }

    pub fn from_vec(src: &[u8]) -> Cmd {
        let mut cmd = Cmd {
            method: Method::Null,
            table: String::new(),
            columns: Vec::new(),
            columns_len: 0,
            values: Vec::new(),
            values_len: 0,
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
                b'@' => cmd.columns_len = data[0] as usize,
                b'^' => cmd.columns.push(String::from_utf8(data.to_vec()).unwrap()),
                b'%' => cmd.values_len = data[0] as usize,
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

    pub fn set_columns(&mut self, columns: Vec<impl ToString>) -> &mut Self {
        self.columns.clear();

        for column in columns {
            self.columns.push(column.to_string());
        }

        self
    }

    fn get_columns(&self) -> String {
        let columns = match self.columns.len() {
            0 => String::from("*"),
            _ => self.columns.join(","),
        };

        columns
    }

    pub fn set_values(&mut self, values: Vec<impl ToString>) -> &mut Self {
        self.values.clear();

        for value in values {
            self.values.push(value.to_string())
        }

        self
    }

    fn get_values(&self) -> String {
        let mut vec = Vec::new();

        for value in &self.values {
            vec.push(format!("'{}'", value));
        }

        vec.join(",")
    }

    pub fn set_condition(&mut self, condition: impl ToString) -> &mut Self {
        self.expression = Some(condition.to_string());

        self
    }

    fn get_condition(&self) -> String {
        let expression = match &self.expression {
            Some(exp) => exp.clone(),
            None => String::new(),
        };

        expression
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

    pub fn query_execution(&self) -> String {
        format!(
            "SELECT {} FROM {} {}",
            self.get_columns(),
            self.table,
            self.get_condition(),
        )
    }

    pub fn insert_execution(&self) -> String {
        format!(
            "INSERT INTO {}({}) values({})",
            self.table,
            self.get_columns(),
            self.get_values()
        )
    }

    pub fn update_execution(&self) -> String {
        let mut execute = Vec::new();

        for i in 0..self.columns_len {
            let column = &self.columns[i];
            let value = &self.values[i];

            execute.push(format!("{} = '{}'", column, value));
        }

        format!(
            "UPDATE {} SET {} {}",
            self.table,
            execute.join(","),
            self.get_condition()
        )
    }

    pub fn delete_execution(&self) -> String {
        format!("DELETE FROM {} {}", self.table, self.get_condition())
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

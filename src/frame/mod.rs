use bytes::Bytes;

#[derive(Debug)]
pub enum Frame {
    Sign(u8),
    Bulk(Bytes),
    Interger(u8),
    Array(Vec<Frame>),
}

impl Frame {
    pub fn array() -> Frame {
        Frame::Array(Vec::new())
    }

    pub fn push_Sign(&mut self, sign: u8) {
        match self {
            Frame::Array(vec) => {
                vec.push(Frame::Sign(sign));
                vec.push(Frame::Bulk(Bytes::from("/r/n")));
            }

            _ => panic!("not array frame"),
        }
    }

    pub fn push_table(&mut self, table: impl ToString) {
        match self {
            Frame::Array(vec) => {
                vec.push(Frame::Sign(b'#'));
                vec.push(Frame::Bulk(Bytes::from(table.to_string())));
                vec.push(Frame::Bulk(Bytes::from("/r/n")));
            }
            _ => panic!("not array frame"),
        }
    }

    pub fn push_columns(&mut self, columns: &Vec<String>) {
        match self {
            Frame::Array(vec) => {
                let len = columns.len() as u8;
                vec.push(Frame::Sign(b'@'));
                vec.push(Frame::Interger(len));
                vec.push(Frame::Bulk(Bytes::from("/r/n")));

                for column in columns {
                    self.push_column(column.to_string());
                }
            }
            _ => panic!("not array frame"),
        }
    }
    fn push_column(&mut self, column: String) {
        if let Frame::Array(vec) = self {
            vec.push(Frame::Sign(b'^'));
            vec.push(Frame::Bulk(Bytes::from(column)));
            vec.push(Frame::Bulk(Bytes::from("/r/n")));
        }
    }
    pub fn push_values(&mut self, values: &Vec<String>) {
        match self {
            Frame::Array(vec) => {
                let len = values.len() as u8;
                vec.push(Frame::Sign(b'%'));
                vec.push(Frame::Interger(len));
                vec.push(Frame::Bulk(Bytes::from("/r/n")));

                for value in values {
                    self.push_value(value.to_string());
                }
            }
            _ => panic!("not array farme"),
        }
    }

    fn push_value(&mut self, value: String) {
        if let Frame::Array(vec) = self {
            vec.push(Frame::Sign(b'!'));
            vec.push(Frame::Bulk(Bytes::from(value)));
            vec.push(Frame::Bulk(Bytes::from("/r/n")));
        }
    }

    pub fn push_expression(&mut self, expression: &Option<String>) {
        if let Some(expression) = expression {
            match self {
                Frame::Array(vec) => {
                    vec.push(Frame::Sign(b'?'));
                    vec.push(Frame::Bulk(Bytes::from(expression.to_string())));
                    vec.push(Frame::Bulk(Bytes::from("/r/n")));
                }
                _ => panic!("not array frame"),
            }
        }
    }
}

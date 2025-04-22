use crate::frame::Frame;

pub enum Method {
    Query,
    Insert,
    Update,
    Delete,
}

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
                frame.push_Sign(b'&');
            }
            Method::Insert => {
                frame.push_Sign(b'+');
            }
            Method::Update => {
                frame.push_Sign(b'*');
            }
            Method::Delete => {
                frame.push_Sign(b'-');
            }
        }

        frame.push_table(&self.table);
        frame.push_columns(&self.columns);
        frame.push_values(&self.values);
        frame.push_expression(&self.expression);

        frame
    }
}

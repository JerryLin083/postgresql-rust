Practice using Rust Tokio to connect to PostgreSQL through TLS,
and use a customized frame for server-client conmmunication.

frame design:
- method\r\n  
	(&: query, +: insert, *: update, -: delete)
- table\r\n 
	(#)
- columns len\r\n 
	(@)
- column\r\n  
	(^)
- values len \r\n 
	(%)
- value\r\n   
	(!)
- expression\r\n  
	(?)


ex: 
	query: &\r\n #table\r\n @len\r\n ^column\r\n^column\r\n ?expression
	insert +\r\n #table\r\n @len\r\n ^column\r\n^column\r\n %len\r\n !value\r\n!value\r\n ?expression\r\n
	update *\r\n #table\r\n @len\r\n ^column\r\n^column\r\n %len\r\n !values\r\n!value\r\n ?expresssion\r\n
	delete -\r\n #table\r\n @len\r\n ?expression\r\n

ToolError -> ErrorData conversion

* We are in progress on switching from mcp_core's ToolError to ErrorData from the rmcp crate
* Review the most recent commit via git show HEAD and then complete more conversions in exactly the same way I did in that commit, across all places needed in the codebase
* Use good judgement when deciding what code to pass to ErrorData::new and make it capture the spirit of the original ToolError
* Work file by file on the replacements, but do not try to compile the project until you think all usages are replaced
* Then compile by running cargo build and verify that works

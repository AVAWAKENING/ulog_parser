use std::collections::BTreeMap;
use std::path::PathBuf;

use evalexpr::*;
use px4_ulog::full_parser::{self, MultiId, SomeVec};

/// 将 time_utc_usec（微秒级 UTC 时间戳）转换为北京时间字符串 (YYYY-MM-DD HH:MM:SS.ffffff)
fn usec_to_beijing_time(usec: u64) -> String {
    let total_sec = usec / 1_000_000;
    let micro = usec % 1_000_000;
    // 加 8 小时转换为北京时间
    let beijing_sec = total_sec + 8 * 3600;
    let days = beijing_sec / 86400;
    let time_of_day = beijing_sec % 86400;
    let hour = time_of_day / 3600;
    let minute = (time_of_day % 3600) / 60;
    let second = time_of_day % 60;
    let (year, month, day) = days_to_ymd(days);
    format!(
        "{:04}-{:02}-{:02} {:02}:{:02}:{:02}.{:06}",
        year, month, day, hour, minute, second, micro
    )
}

/// 将自 1970-01-01 以来的天数转换为 (year, month, day)
fn days_to_ymd(mut days: u64) -> (u64, u64, u64) {
    let mut year = 1970u64;
    loop {
        let dy = if is_leap(year) { 366 } else { 365 };
        if days < dy {
            break;
        }
        days -= dy;
        year += 1;
    }
    let leap = is_leap(year);
    let month_days: [u64; 12] = if leap {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };
    let mut month = 1u64;
    for &md in &month_days {
        if days < md {
            break;
        }
        days -= md;
        month += 1;
    }
    (year, month, days + 1)
}

fn is_leap(y: u64) -> bool {
    (y % 4 == 0 && y % 100 != 0) || y % 400 == 0
}

/// 一条规则：输出字段名 = 表达式
#[derive(Debug, Clone)]
pub struct Rule {
    /// 输出字段名（等号左边）
    pub output_name: String,
    /// 原始表达式文本（等号右边）
    pub expression: String,
    /// 是否为简单字段引用（仅含字母、数字、下划线，无运算符）
    pub is_simple_ref: bool,
}

/// 规则集合：消息名 -> 规则列表
pub type Rules = BTreeMap<String, Vec<Rule>>;

/// 判断表达式是否为简单字段引用
fn is_simple_field_ref(expr: &str) -> bool {
    !expr.is_empty()
        && expr
            .chars()
            .next()
            .map(|c| c.is_alphabetic() || c == '_')
            .unwrap_or(false)
        && expr.chars().all(|c| c.is_alphanumeric() || c == '_')
}

/// 解析 rules.txt 文件
///
/// 文件格式：
/// ```text
/// message_name:
///     output_field=expression
///     another_field=some_var
/// ```
///
/// 以 `#` 开头的行为注释，空行被忽略。
/// 消息头行以 `:` 结尾，后续缩进行为该消息的规则。
pub fn parse_rules_file(path: &std::path::Path) -> Result<Rules, String> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("无法读取规则文件 {}: {}", path.display(), e))?;

    let mut rules: Rules = BTreeMap::new();
    let mut current_msg: Option<String> = None;

    for line in content.lines() {
        let trimmed = line.trim();
        // 跳过空行和注释
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        if trimmed.ends_with(':') {
            // 消息头行
            let msg_name = trimmed.trim_end_matches(':').trim().to_string();
            current_msg = Some(msg_name);
        } else if let Some(msg) = &current_msg {
            // 规则行：输出字段名=表达式
            if let Some((output_name, expression)) = trimmed.split_once('=') {
                let output_name = output_name.trim().to_string();
                let expression = expression.trim().to_string();
                rules.entry(msg.clone()).or_default().push(Rule {
                    output_name,
                    is_simple_ref: is_simple_field_ref(&expression),
                    expression,
                });
            }
        }
    }

    Ok(rules)
}

/// 获取可执行文件目录下的 rules.txt 路径
pub fn get_rules_path() -> PathBuf {
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(parent) = exe_path.parent() {
            return parent.join("rules.txt");
        }
    }
    PathBuf::from("rules.txt")
}

/// 扫描源目录，找到所有 .ulg / .ulog 文件
pub fn find_ulog_files(dir: &std::path::Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
                if ext == "ulg" || ext == "ulog" {
                    files.push(path);
                }
            }
        }
    }
    files.sort();
    files
}

/// 将 SomeVec 中的第 i 个值格式化为字符串
pub fn some_vec_to_string(vec: &SomeVec, i: usize) -> String {
    match vec {
        SomeVec::Int8(v) => v.get(i).map(|x| x.to_string()),
        SomeVec::UInt8(v) => v.get(i).map(|x| x.to_string()),
        SomeVec::Int16(v) => v.get(i).map(|x| x.to_string()),
        SomeVec::UInt16(v) => v.get(i).map(|x| x.to_string()),
        SomeVec::Int32(v) => v.get(i).map(|x| x.to_string()),
        SomeVec::UInt32(v) => v.get(i).map(|x| x.to_string()),
        SomeVec::Int64(v) => v.get(i).map(|x| x.to_string()),
        SomeVec::UInt64(v) => v.get(i).map(|x| x.to_string()),
        SomeVec::Float(v) => v.get(i).map(|x| format!("{:.6}", x)),
        SomeVec::Double(v) => v.get(i).map(|x| format!("{:.10}", x)),
        SomeVec::Bool(v) => v.get(i).map(|x| x.to_string()),
        SomeVec::Char(v) => v.get(i).map(|x| x.to_string()),
    }
    .unwrap_or_default()
}

/// 获取 SomeVec 的长度（采样点数）
pub fn some_vec_len(vec: &SomeVec) -> usize {
    match vec {
        SomeVec::Int8(v) => v.len(),
        SomeVec::UInt8(v) => v.len(),
        SomeVec::Int16(v) => v.len(),
        SomeVec::UInt16(v) => v.len(),
        SomeVec::Int32(v) => v.len(),
        SomeVec::UInt32(v) => v.len(),
        SomeVec::Int64(v) => v.len(),
        SomeVec::UInt64(v) => v.len(),
        SomeVec::Float(v) => v.len(),
        SomeVec::Double(v) => v.len(),
        SomeVec::Bool(v) => v.len(),
        SomeVec::Char(v) => v.len(),
    }
}

/// 根据 rules.txt 规则导出 ulog 文件到 CSV
///
/// 每个 (ulog文件, 消息) 组合生成一个 CSV 文件，文件名为 `{ulog_stem}_{message_name}.csv`。
/// 表达式中可使用 evalexpr 支持的数学函数（sqrt, sin, cos 等）和 `^` 运算符。
/// 内置自定义函数 `beijing_time(x)` 可将微秒 UTC 时间戳转换为北京时间字符串。
pub fn export_ulog_with_rules(
    path: &std::path::Path,
    dest_dir: &std::path::Path,
    rules: &Rules,
) -> Result<Vec<String>, String> {
    let path_str = path
        .to_str()
        .ok_or_else(|| "路径包含非法字符".to_string())?;
    let parsed = full_parser::read_file(path_str).map_err(|e| format!("解析失败: {}", e))?;

    let stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("output");
    let mut generated_files = Vec::new();

    for (msg_name, rule_list) in rules {
        // 获取消息数据
        let multi_map = match parsed.messages.get(msg_name) {
            Some(m) => m,
            None => continue,
        };
        let fields = match multi_map.get(&MultiId::new(0)) {
            Some(f) => f,
            None => continue,
        };

        // 获取行数
        let row_count = fields.values().map(some_vec_len).max().unwrap_or(0);
        if row_count == 0 {
            continue;
        }

        // 创建 CSV 文件
        let csv_filename = format!("{}_{}.csv", stem, msg_name);
        let csv_path = dest_dir.join(&csv_filename);
        let mut wtr =
            csv::Writer::from_path(&csv_path).map_err(|e| format!("无法创建 CSV 文件: {}", e))?;

        // 写入表头
        let headers: Vec<&str> = rule_list.iter().map(|r| r.output_name.as_str()).collect();
        wtr.write_record(&headers)
            .map_err(|e| format!("写入表头失败: {}", e))?;

        // 预编译非简单引用的表达式
        let compiled: Vec<Option<Node>> = rule_list
            .iter()
            .map(|rule| {
                if rule.is_simple_ref {
                    None
                } else {
                    evalexpr::build_operator_tree(&rule.expression).ok()
                }
            })
            .collect();

        // 构建 evalexpr 上下文，注册 beijing_time 自定义函数
        let fn_beijing_time = Function::new(|argument| {
            if let Ok(f) = argument.as_float() {
                Ok(Value::String(usec_to_beijing_time(f as u64)))
            } else if let Ok(i) = argument.as_int() {
                Ok(Value::String(usec_to_beijing_time(i as u64)))
            } else {
                Err(EvalexprError::expected_number(argument.clone()))
            }
        });

        // 处理每一行
        for i in 0..row_count {
            let mut context = HashMapContext::new();
            let _ = context.set_function("beijing_time".into(), fn_beijing_time.clone());

            // 将消息中所有字段的值填入上下文
            for (field_name, some_vec) in fields {
                let val_str = some_vec_to_string(some_vec, i);
                if let Ok(val) = val_str.parse::<f64>() {
                    let _ = context.set_value(field_name.clone().into(), val.into());
                }
            }

            // 评估每条规则
            let mut row: Vec<String> = Vec::with_capacity(rule_list.len());
            for (j, rule) in rule_list.iter().enumerate() {
                if rule.is_simple_ref {
                    // 简单字段引用：直接使用原始字符串值（避免大整数精度丢失）
                    let val = fields
                        .get(&rule.expression)
                        .map(|v| some_vec_to_string(v, i))
                        .unwrap_or_default();
                    row.push(val);
                } else if let Some(ref node) = compiled[j] {
                    match node.eval_with_context(&context) {
                        Ok(Value::Float(f)) => row.push(format!("{:.6}", f)),
                        Ok(Value::Int(n)) => row.push(n.to_string()),
                        Ok(Value::String(s)) => row.push(s),
                        Ok(v) => row.push(v.to_string()),
                        Err(_) => row.push(String::new()),
                    }
                } else {
                    row.push(String::new());
                }
            }

            wtr.write_record(&row)
                .map_err(|e| format!("写入数据行失败: {}", e))?;
        }

        wtr.flush().map_err(|e| format!("刷新 CSV 失败: {}", e))?;
        generated_files.push(csv_path.display().to_string());
    }

    Ok(generated_files)
}

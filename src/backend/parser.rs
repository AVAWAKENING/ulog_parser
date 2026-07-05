use std::collections::BTreeMap;
use std::path::PathBuf;

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
    // 从 1970-01-01 计算年月日
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

/// "消息名&字段名" 的键，如 "vehicle_gps_position&lat"
pub type FieldKey = String;

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

/// 解析一个 ulog 文件，提取所有消息名&字段名
pub fn extract_field_keys(path: &std::path::Path) -> Result<BTreeMap<String, Vec<String>>, String> {
    let path_str = path
        .to_str()
        .ok_or_else(|| "路径包含非法字符".to_string())?;
    let parsed = full_parser::read_file(path_str).map_err(|e| format!("解析失败: {}", e))?;

    let mut keys: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for (msg_name, multi_map) in &parsed.messages {
        // 跳过 "event" 消息
        if msg_name == "event" {
            continue;
        }
        // 取第一个实例 (MultiId=0) 来获取字段列表
        if let Some(fields) = multi_map.get(&MultiId::new(0)) {
            let mut field_names: Vec<String> = fields.keys().cloned().collect();
            field_names.sort();
            keys.insert(msg_name.clone(), field_names);
        }
    }
    Ok(keys)
}

/// 合并字段 keys（取并集）
pub fn merge_field_keys(
    base: &mut BTreeMap<String, Vec<String>>,
    new: BTreeMap<String, Vec<String>>,
) {
    for (msg, fields) in new {
        let entry = base.entry(msg).or_default();
        for f in fields {
            if !entry.contains(&f) {
                entry.push(f);
            }
        }
        entry.sort();
    }
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

/// 导出一个 ulog 文件的选中字段到 CSV
pub fn export_ulog_to_csv(
    path: &std::path::Path,
    dest_dir: &std::path::Path,
    selected: &[FieldKey],
) -> Result<String, String> {
    let path_str = path
        .to_str()
        .ok_or_else(|| "路径包含非法字符".to_string())?;
    let parsed = full_parser::read_file(path_str).map_err(|e| format!("解析失败: {}", e))?;

    // 按消息名分组选中的字段
    let mut msg_fields: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for key in selected {
        let parts: Vec<&str> = key.splitn(2, '&').collect();
        if parts.len() == 2 {
            let entry = msg_fields.entry(parts[0].to_string()).or_default();
            if !entry.contains(&parts[1].to_string()) {
                entry.push(parts[1].to_string());
            }
        }
    }

    // 生成 CSV 文件名
    let stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("output");
    let csv_path = dest_dir.join(format!("{}.csv", stem));

    let mut wtr =
        csv::Writer::from_path(&csv_path).map_err(|e| format!("无法创建 CSV 文件: {}", e))?;

    // 写表头：timestamp + 各选中字段的全名（字段名&消息名），按选择顺序排列
    let mut headers = vec!["timestamp".to_string()];
    let header_display: Vec<String> = selected
        .iter()
        .map(|k| {
            let parts: Vec<&str> = k.splitn(2, '&').collect();
            if parts.len() == 2 {
                format!("{}&{}", parts[1], parts[0])
            } else {
                k.clone()
            }
        })
        .collect();
    headers.extend(header_display.iter().cloned());

    // 记录 time_utc_usec 列位置及其对应的北京时间列位置
    let mut utc_usec_cols: Vec<(usize, usize)> = Vec::new();
    {
        // 先收集所有 time_utc_usec 列的索引（从后往前插入，避免索引偏移）
        let mut utc_indices: Vec<usize> = Vec::new();
        for (idx, h) in headers.iter().enumerate() {
            if h.starts_with("time_utc_usec&") {
                utc_indices.push(idx);
            }
        }
        // 从后往前插入北京时间列
        for &utc_idx in utc_indices.iter().rev() {
            let beijing_header = format!(
                "time_utc_usec_beijing&{}",
                headers[utc_idx].split('&').nth(1).unwrap_or("")
            );
            headers.insert(utc_idx + 1, beijing_header);
        }
        // 插入完成后，重新收集 (utc_col, beijing_col) 位置对
        let mut i = 0;
        while i < headers.len() {
            if headers[i].starts_with("time_utc_usec&") && i + 1 < headers.len() {
                utc_usec_cols.push((i, i + 1));
                i += 2;
            } else {
                i += 1;
            }
        }
    }

    // 记录 ground_speed 计算列的位置：(vn_m_s列, ve_m_s列, ground_speed列)
    let mut ground_speed_cols: Vec<(usize, usize, usize)> = Vec::new();
    {
        // 找出同时包含 vn_m_s 和 ve_m_s 的消息
        let mut msg_has_vn_ve: Vec<String> = Vec::new();
        for (msg_name, field_names) in &msg_fields {
            if field_names.iter().any(|f| f == "vn_m_s") && field_names.iter().any(|f| f == "ve_m_s") {
                msg_has_vn_ve.push(msg_name.clone());
            }
        }
        // 从后往前为这些消息插入 ground_speed 列（在 ve_m_s 之后）
        let mut ve_indices: Vec<usize> = Vec::new();
        for msg in &msg_has_vn_ve {
            let ve_header = format!("ve_m_s&{}", msg);
            if let Some(idx) = headers.iter().position(|h| h == &ve_header) {
                ve_indices.push(idx);
            }
        }
        for &ve_idx in ve_indices.iter().rev() {
            let msg_name = headers[ve_idx].split('&').nth(1).unwrap_or("");
            let gs_header = format!("ground_speed&{}", msg_name);
            headers.insert(ve_idx + 1, gs_header);
        }
        // 收集最终列位置
        for msg in &msg_has_vn_ve {
            let vn_header = format!("vn_m_s&{}", msg);
            let ve_header = format!("ve_m_s&{}", msg);
            let gs_header = format!("ground_speed&{}", msg);
            if let (Some(vn_col), Some(ve_col), Some(gs_col)) = (
                headers.iter().position(|h| h == &vn_header),
                headers.iter().position(|h| h == &ve_header),
                headers.iter().position(|h| h == &gs_header),
            ) {
                ground_speed_cols.push((vn_col, ve_col, gs_col));
            }
        }
    }

    wtr.write_record(&headers)
        .map_err(|e| format!("写入表头失败: {}", e))?;

    // 收集所有数据行，按 timestamp 排序
    // 每行: (timestamp, [field_values...])
    let mut rows: Vec<(u64, Vec<String>)> = Vec::new();

    for (msg_name, field_names) in &msg_fields {
        let multi_map = match parsed.messages.get(msg_name) {
            Some(m) => m,
            None => continue,
        };
        // 只取 MultiId=0 的实例
        let fields = match multi_map.get(&MultiId::new(0)) {
            Some(f) => f,
            None => continue,
        };

        // 获取行数
        let row_count = field_names
            .iter()
            .filter_map(|f| fields.get(f))
            .map(some_vec_len)
            .max()
            .unwrap_or(0);

        // 获取 timestamp 列
        let timestamps: Vec<u64> = match fields.get("timestamp") {
            Some(SomeVec::UInt64(v)) => v.clone(),
            Some(SomeVec::Int64(v)) => v.iter().map(|&x| x as u64).collect(),
            _ => (0..row_count as u64).collect(),
        };

        // 检查此消息是否包含 time_utc_usec 字段
        let has_utc_usec = field_names.iter().any(|f| f == "time_utc_usec");
        let utc_usec_vec: Option<&SomeVec> = if has_utc_usec {
            fields.get("time_utc_usec")
        } else {
            None
        };

        for i in 0..row_count {
            let ts = timestamps.get(i).copied().unwrap_or(i as u64);

            // 如果 rows 还没有这一行（按 timestamp），就初始化
            let row_idx = rows.iter().position(|(t, _)| *t == ts);
            match row_idx {
                Some(idx) => {
                    // 填充这个消息的字段值
                    for field_name in field_names {
                        let header_name = format!("{}&{}", field_name, msg_name);
                        let col_idx = headers.iter().position(|h| h == &header_name).unwrap();
                        if let Some(some_vec) = fields.get(field_name) {
                            rows[idx].1[col_idx] = some_vec_to_string(some_vec, i);
                        }
                    }
                    // 如果有 time_utc_usec，填充北京时间列
                    if let Some(some_vec) = utc_usec_vec {
                        let utc_usec_val = match some_vec {
                            SomeVec::UInt64(v) => v.get(i).copied(),
                            SomeVec::Int64(v) => v.get(i).map(|&x| x as u64),
                            SomeVec::UInt32(v) => v.get(i).map(|&x| x as u64),
                            SomeVec::Int32(v) => v.get(i).map(|&x| x as u64),
                            _ => None,
                        };
                        if let Some(usec) = utc_usec_val {
                            let header_name = format!("time_utc_usec&{}", msg_name);
                            if let Some(utc_col) = headers.iter().position(|h| h == &header_name) {
                                for &(utc_idx, bj_idx) in &utc_usec_cols {
                                    if utc_idx == utc_col {
                                        rows[idx].1[bj_idx] = usec_to_beijing_time(usec);
                                        break;
                                    }
                                }
                            }
                        }
                    }
                    // 如果有 vn_m_s 和 ve_m_s，计算并填充 ground_speed 列
                    for &(vn_col, ve_col, gs_col) in &ground_speed_cols {
                        let vn_str = &rows[idx].1[vn_col];
                        let ve_str = &rows[idx].1[ve_col];
                        if let (Ok(vn), Ok(ve)) = (vn_str.parse::<f64>(), ve_str.parse::<f64>()) {
                            rows[idx].1[gs_col] = format!("{:.6}", (vn * vn + ve * ve).sqrt());
                        }
                    }
                }
                None => {
                    let mut row = vec![String::new(); headers.len()];
                    row[0] = ts.to_string();
                    for field_name in field_names {
                        let header_name = format!("{}&{}", field_name, msg_name);
                        let col_idx = headers.iter().position(|h| h == &header_name).unwrap();
                        if let Some(some_vec) = fields.get(field_name) {
                            row[col_idx] = some_vec_to_string(some_vec, i);
                        }
                    }
                    // 如果有 time_utc_usec，填充北京时间列
                    if let Some(some_vec) = utc_usec_vec {
                        let utc_usec_val = match some_vec {
                            SomeVec::UInt64(v) => v.get(i).copied(),
                            SomeVec::Int64(v) => v.get(i).map(|&x| x as u64),
                            SomeVec::UInt32(v) => v.get(i).map(|&x| x as u64),
                            SomeVec::Int32(v) => v.get(i).map(|&x| x as u64),
                            _ => None,
                        };
                        if let Some(usec) = utc_usec_val {
                            let header_name = format!("time_utc_usec&{}", msg_name);
                            if let Some(utc_col) = headers.iter().position(|h| h == &header_name) {
                                for &(utc_idx, bj_idx) in &utc_usec_cols {
                                    if utc_idx == utc_col {
                                        row[bj_idx] = usec_to_beijing_time(usec);
                                        break;
                                    }
                                }
                            }
                        }
                    }
                    // 如果有 vn_m_s 和 ve_m_s，计算并填充 ground_speed 列
                    for &(vn_col, ve_col, gs_col) in &ground_speed_cols {
                        let vn_str = &row[vn_col];
                        let ve_str = &row[ve_col];
                        if let (Ok(vn), Ok(ve)) = (vn_str.parse::<f64>(), ve_str.parse::<f64>()) {
                            row[gs_col] = format!("{:.6}", (vn * vn + ve * ve).sqrt());
                        }
                    }
                    rows.push((ts, row));
                }
            }
        }
    }

    // 按 timestamp 排序后写入
    rows.sort_by_key(|(ts, _)| *ts);
    for (_, row) in rows {
        wtr.write_record(&row)
            .map_err(|e| format!("写入数据行失败: {}", e))?;
    }

    wtr.flush().map_err(|e| format!("刷新 CSV 失败: {}", e))?;
    Ok(csv_path.display().to_string())
}

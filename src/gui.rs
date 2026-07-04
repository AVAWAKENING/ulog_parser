use std::collections::BTreeMap;
use std::path::PathBuf;

use crate::backend::{self, FieldKey};

const DARK_BG: egui::Color32 = egui::Color32::from_rgb(30, 30, 35);
const CARD_BG: egui::Color32 = egui::Color32::from_rgb(45, 45, 50);
const TEXT_COLOR: egui::Color32 = egui::Color32::from_rgb(220, 220, 225);
const LABEL_COLOR: egui::Color32 = egui::Color32::from_rgb(160, 160, 170);
const ACCENT_COLOR: egui::Color32 = egui::Color32::from_rgb(100, 180, 255);
const GREEN_BTN: egui::Color32 = egui::Color32::from_rgb(46, 125, 50);
const BLUE_BTN: egui::Color32 = egui::Color32::from_rgb(30, 90, 180);
const SELECTED_BTN: egui::Color32 = egui::Color32::from_rgb(200, 80, 40);

pub struct UlogerApp {
    source_dir: PathBuf,
    dest_dir: PathBuf,
    status_log: String,

    /// 解析后得到的所有 字段键（消息名&字段名），按消息名分组排序
    field_keys: BTreeMap<String, Vec<String>>, // msg_name -> [field_name]
    /// 用户选中的字段键，按选择顺序排列
    selected_fields: Vec<FieldKey>,
}

impl UlogerApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            source_dir: PathBuf::new(),
            dest_dir: PathBuf::new(),
            status_log: "就绪".to_string(),
            field_keys: BTreeMap::new(),
            selected_fields: Vec::new(),
        }
    }

    fn pick_directory() -> Option<PathBuf> {
        rfd::FileDialog::new().pick_folder()
    }

    fn log(&mut self, msg: &str) {
        self.status_log = msg.to_string();
    }
}

impl eframe::App for UlogerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            // 外层滚动区域，保证内容过多时整个界面可滚动
            egui::ScrollArea::vertical()
                .id_salt("outer_scroll")
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.spacing_mut().item_spacing = egui::vec2(8.0, 8.0);

                    egui::Frame::none().fill(DARK_BG).show(ui, |ui| {
                        ui.add_space(8.0);

                        // 1. 源目录选择 + 解析按钮
                        egui::Frame::none()
                            .fill(CARD_BG)
                            .rounding(8.0)
                            .inner_margin(12.0)
                            .show(ui, |ui| {
                                ui.set_min_width(ui.available_width());
                                ui.heading(egui::RichText::new("源目录").color(TEXT_COLOR));
                                ui.add_space(8.0);
                                ui.separator();
                                ui.add_space(8.0);

                                ui.horizontal(|ui| {
                                    let source_text = if self.source_dir.as_os_str().is_empty() {
                                        "未选择".to_string()
                                    } else {
                                        self.source_dir.display().to_string()
                                    };
                                    ui.label(egui::RichText::new(&source_text).color(LABEL_COLOR));
                                    ui.with_layout(
                                        egui::Layout::right_to_left(egui::Align::Center),
                                        |ui| {
                                            if ui
                                                .add_sized(
                                                    [100.0, 32.0],
                                                    egui::Button::new("解析").fill(GREEN_BTN),
                                                )
                                                .clicked()
                                            {
                                                if self.source_dir.as_os_str().is_empty() {
                                                    self.log("请先选择源目录");
                                                } else {
                                                    let ulog_files =
                                                        backend::find_ulog_files(&self.source_dir);
                                                    if ulog_files.is_empty() {
                                                        self.log("源目录中未找到 .ulg/.ulog 文件");
                                                    } else {
                                                        self.field_keys.clear();
                                                        self.selected_fields.clear();
                                                        let mut error_count = 0;
                                                        let mut ok_count = 0;
                                                        for f in &ulog_files {
                                                            match backend::extract_field_keys(f) {
                                                                Ok(keys) => {
                                                                    backend::merge_field_keys(
                                                                        &mut self.field_keys,
                                                                        keys,
                                                                    );
                                                                    ok_count += 1;
                                                                }
                                                                Err(e) => {
                                                                    log::warn!(
                                                                        "解析 {} 失败: {}",
                                                                        f.display(),
                                                                        e
                                                                    );
                                                                    error_count += 1;
                                                                }
                                                            }
                                                        }
                                                        let total_fields: usize = self
                                                            .field_keys
                                                            .values()
                                                            .map(|v| v.len())
                                                            .sum();
                                                        self.log(&format!(
                                                            "解析完成: {} 个文件成功, {} 个失败, 共 {} 个消息, {} 个字段",
                                                            ok_count,
                                                            error_count,
                                                            self.field_keys.len(),
                                                            total_fields
                                                        ));
                                                    }
                                                }
                                            }
                                            ui.add_space(4.0);
                                            if ui
                                                .add_sized(
                                                    [100.0, 32.0],
                                                    egui::Button::new("浏览...").fill(BLUE_BTN),
                                                )
                                                .clicked()
                                            {
                                                if let Some(path) = Self::pick_directory() {
                                                    self.source_dir = path.clone();
                                                    self.field_keys.clear();
                                                    self.selected_fields.clear();
                                                    self.log(&format!("源目录: {}", path.display()));
                                                }
                                            }
                                        },
                                    );
                                });
                            });

                        ui.add_space(4.0);

                        // 2. 目标目录选择
                        egui::Frame::none()
                            .fill(CARD_BG)
                            .rounding(8.0)
                            .inner_margin(12.0)
                            .show(ui, |ui| {
                                ui.set_min_width(ui.available_width());
                                ui.heading(egui::RichText::new("目标目录").color(TEXT_COLOR));
                                ui.add_space(8.0);
                                ui.separator();
                                ui.add_space(8.0);

                                ui.horizontal(|ui| {
                                    let dest_text = if self.dest_dir.as_os_str().is_empty() {
                                        "未选择".to_string()
                                    } else {
                                        self.dest_dir.display().to_string()
                                    };
                                    ui.label(egui::RichText::new(&dest_text).color(LABEL_COLOR));
                                    ui.with_layout(
                                        egui::Layout::right_to_left(egui::Align::Center),
                                        |ui| {
                                            if ui
                                                .add_sized(
                                                    [100.0, 32.0],
                                                    egui::Button::new("浏览...").fill(BLUE_BTN),
                                                )
                                                .clicked()
                                            {
                                                if let Some(path) = Self::pick_directory() {
                                                    self.dest_dir = path.clone();
                                                    self.log(&format!("目标目录: {}", path.display()));
                                                }
                                            }
                                        },
                                    );
                                });
                            });

                        ui.add_space(4.0);

                        // 3. 字段选择区域
                        egui::Frame::none()
                            .fill(CARD_BG)
                            .rounding(8.0)
                            .inner_margin(12.0)
                            .show(ui, |ui| {
                                ui.set_min_width(ui.available_width());
                                ui.heading(egui::RichText::new("字段选择").color(TEXT_COLOR));
                                ui.add_space(8.0);
                                ui.separator();
                                ui.add_space(8.0);

                                if self.field_keys.is_empty() {
                                    ui.horizontal(|ui| {
                                        ui.label(
                                            egui::RichText::new("请先选择源目录并点击「解析」")
                                                .color(LABEL_COLOR),
                                        );
                                    });
                                } else {
                                    // 全选 / 全不选 按钮
                                    ui.horizontal(|ui| {
                                        let all_keys: Vec<FieldKey> = self
                                            .field_keys
                                            .iter()
                                            .flat_map(|(msg, fields)| {
                                                fields.iter().map(move |f| format!("{}&{}", msg, f))
                                            })
                                            .collect();

                                        if ui
                                            .add_sized(
                                                [80.0, 28.0],
                                                egui::Button::new("全选").fill(BLUE_BTN),
                                            )
                                            .clicked()
                                        {
                                            for k in &all_keys {
                                                if !self.selected_fields.contains(k) {
                                                    self.selected_fields.push(k.clone());
                                                }
                                            }
                                        }
                                        if ui
                                            .add_sized(
                                                [80.0, 28.0],
                                                egui::Button::new("全不选").fill(BLUE_BTN),
                                            )
                                            .clicked()
                                        {
                                            self.selected_fields.clear();
                                        }
                                        ui.label(
                                            egui::RichText::new(&format!(
                                                "已选 {} / {} 个字段",
                                                self.selected_fields.len(),
                                                all_keys.len()
                                            ))
                                            .color(LABEL_COLOR),
                                        );
                                    });
                                    ui.add_space(4.0);

                                    // 按消息名分组显示字段选择按钮
                                    egui::ScrollArea::vertical()
                                        .id_salt("field_scroll")
                                        .max_height(400.0)
                                        .auto_shrink([false, false])
                                        .show(ui, |ui| {
                                            for (msg_name, field_names) in &self.field_keys {
                                                ui.label(
                                                    egui::RichText::new(msg_name)
                                                        .color(ACCENT_COLOR)
                                                        .strong(),
                                                );
                                                ui.add_space(4.0);
                                                ui.horizontal_wrapped(|ui| {
                                                    // 增大按钮间距，避免点击区域重叠
                                                    ui.spacing_mut().item_spacing = egui::vec2(6.0, 6.0);
                                                    for field_name in field_names {
                                                        let key =
                                                            format!("{}&{}", msg_name, field_name);
                                                        let is_selected =
                                                            self.selected_fields.contains(&key);
                                                        let btn_color =
                                                            if is_selected { SELECTED_BTN } else { CARD_BG };
                                                        let btn = egui::Button::new(
                                                            egui::RichText::new(field_name)
                                                                .color(if is_selected {
                                                                    TEXT_COLOR
                                                                } else {
                                                                    LABEL_COLOR
                                                                }),
                                                        )
                                                        .min_size(egui::vec2(0.0, 28.0))
                                                        .fill(btn_color)
                                                        .stroke(egui::Stroke::new(
                                                            1.0,
                                                            if is_selected {
                                                                ACCENT_COLOR
                                                            } else {
                                                                LABEL_COLOR
                                                            },
                                                        ));
                                                        let resp = ui.add(btn);
                                                        if resp.clicked() {
                                                            if is_selected {
                                                                self.selected_fields.retain(|k| k != &key);
                                                            } else {
                                                                self.selected_fields.push(key);
                                                            }
                                                        }
                                                    }
                                                });
                                                ui.add_space(8.0);
                                            }
                                        });
                                }
                            });

                        ui.add_space(4.0);

                        // 4. 生成
                        egui::Frame::none()
                            .fill(CARD_BG)
                            .rounding(8.0)
                            .inner_margin(12.0)
                            .show(ui, |ui| {
                                ui.set_min_width(ui.available_width());
                                ui.horizontal(|ui| {
                                    if ui
                                        .add_sized(
                                            [160.0, 36.0],
                                            egui::Button::new("生成").fill(GREEN_BTN),
                                        )
                                        .clicked()
                                    {
                                        if self.source_dir.as_os_str().is_empty() {
                                            self.log("请先选择源目录");
                                        } else if self.dest_dir.as_os_str().is_empty() {
                                            self.log("请先选择目标目录");
                                        } else if self.selected_fields.is_empty() {
                                            self.log("请至少选择一个字段");
                                        } else {
                                            let ulog_files =
                                                backend::find_ulog_files(&self.source_dir);
                                            if ulog_files.is_empty() {
                                                self.log("源目录中未找到 .ulg/.ulog 文件");
                                            } else {
                                                let mut ok = 0;
                                                let mut err = 0;
                                                for f in &ulog_files {
                                                    match backend::export_ulog_to_csv(
                                                        f,
                                                        &self.dest_dir,
                                                        &self.selected_fields,
                                                    ) {
                                                        Ok(csv_path) => {
                                                            log::info!(
                                                                "已生成: {}",
                                                                csv_path
                                                            );
                                                            ok += 1;
                                                        }
                                                        Err(e) => {
                                                            log::error!(
                                                                "导出 {} 失败: {}",
                                                                f.display(),
                                                                e
                                                            );
                                                            err += 1;
                                                        }
                                                    }
                                                }
                                                self.log(&format!(
                                                    "生成完成: {} 个文件成功, {} 个失败",
                                                    ok, err
                                                ));
                                            }
                                        }
                                    }
                                });
                            });

                        ui.add_space(4.0);

                        // 5. 状态信息文字框
                        egui::Frame::none()
                            .fill(CARD_BG)
                            .rounding(8.0)
                            .inner_margin(12.0)
                            .show(ui, |ui| {
                                ui.set_min_width(ui.available_width());
                                ui.set_min_height(80.0);
                                ui.heading(egui::RichText::new("状态信息").color(TEXT_COLOR));
                                ui.add_space(8.0);
                                ui.separator();
                                ui.add_space(8.0);

                                egui::ScrollArea::vertical()
                                    .id_salt("status_scroll")
                                    .max_height(150.0)
                                    .auto_shrink([false, false])
                                    .show(ui, |ui| {
                                        ui.label(
                                            egui::RichText::new(&self.status_log)
                                                .color(ACCENT_COLOR),
                                        );
                                    });
                            });
                    });
                });
        });

        ctx.request_repaint_after(std::time::Duration::from_millis(100));
    }
}

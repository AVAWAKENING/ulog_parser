use std::path::PathBuf;

use crate::backend::{self, Rules};

const DARK_BG: egui::Color32 = egui::Color32::from_rgb(30, 30, 35);
const CARD_BG: egui::Color32 = egui::Color32::from_rgb(45, 45, 50);
const TEXT_COLOR: egui::Color32 = egui::Color32::from_rgb(220, 220, 225);
const LABEL_COLOR: egui::Color32 = egui::Color32::from_rgb(160, 160, 170);
const ACCENT_COLOR: egui::Color32 = egui::Color32::from_rgb(100, 180, 255);
const GREEN_BTN: egui::Color32 = egui::Color32::from_rgb(46, 125, 50);
const BLUE_BTN: egui::Color32 = egui::Color32::from_rgb(30, 90, 180);

pub struct UlogerApp {
    source_dir: PathBuf,
    dest_dir: PathBuf,
    status_log: String,
    /// 已加载的规则
    loaded_rules: Option<Rules>,
}

impl UlogerApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            source_dir: PathBuf::new(),
            dest_dir: PathBuf::new(),
            status_log: "就绪".to_string(),
            loaded_rules: None,
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
            egui::ScrollArea::vertical()
                .id_salt("outer_scroll")
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.spacing_mut().item_spacing = egui::vec2(8.0, 8.0);

                    egui::Frame::none().fill(DARK_BG).show(ui, |ui| {
                        ui.add_space(8.0);

                        // 1. 源目录选择
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
                                    ui.label(
                                        egui::RichText::new(&source_text).color(LABEL_COLOR),
                                    );
                                    ui.with_layout(
                                        egui::Layout::right_to_left(egui::Align::Center),
                                        |ui| {
                                            if ui
                                                .add_sized(
                                                    [100.0, 32.0],
                                                    egui::Button::new("浏览...")
                                                        .fill(BLUE_BTN),
                                                )
                                                .clicked()
                                            {
                                                if let Some(path) = Self::pick_directory() {
                                                    self.source_dir = path.clone();
                                                    self.log(&format!(
                                                        "源目录: {}",
                                                        path.display()
                                                    ));
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
                                    ui.label(
                                        egui::RichText::new(&dest_text).color(LABEL_COLOR),
                                    );
                                    ui.with_layout(
                                        egui::Layout::right_to_left(egui::Align::Center),
                                        |ui| {
                                            if ui
                                                .add_sized(
                                                    [100.0, 32.0],
                                                    egui::Button::new("浏览...")
                                                        .fill(BLUE_BTN),
                                                )
                                                .clicked()
                                            {
                                                if let Some(path) = Self::pick_directory() {
                                                    self.dest_dir = path.clone();
                                                    self.log(&format!(
                                                        "目标目录: {}",
                                                        path.display()
                                                    ));
                                                }
                                            }
                                        },
                                    );
                                });
                            });

                        ui.add_space(4.0);

                        // 3. 规则文件显示
                        egui::Frame::none()
                            .fill(CARD_BG)
                            .rounding(8.0)
                            .inner_margin(12.0)
                            .show(ui, |ui| {
                                ui.set_min_width(ui.available_width());
                                ui.heading(egui::RichText::new("规则文件").color(TEXT_COLOR));
                                ui.add_space(8.0);
                                ui.separator();
                                ui.add_space(8.0);

                                let rules_path = backend::get_rules_path();
                                ui.horizontal(|ui| {
                                    ui.label(
                                        egui::RichText::new(format!(
                                            "路径: {}",
                                            rules_path.display()
                                        ))
                                        .color(LABEL_COLOR),
                                    );
                                    ui.with_layout(
                                        egui::Layout::right_to_left(egui::Align::Center),
                                        |ui| {
                                            if ui
                                                .add_sized(
                                                    [100.0, 32.0],
                                                    egui::Button::new("加载规则")
                                                        .fill(BLUE_BTN),
                                                )
                                                .clicked()
                                            {
                                                if rules_path.exists() {
                                                    match backend::parse_rules_file(&rules_path) {
                                                        Ok(rules) => {
                                                            let msg_count = rules.len();
                                                            let rule_count: usize = rules
                                                                .values()
                                                                .map(|v| v.len())
                                                                .sum();
                                                            self.loaded_rules = Some(rules);
                                                            self.log(&format!(
                                                                "规则加载成功: {} 个消息, {} 条规则",
                                                                msg_count, rule_count
                                                            ));
                                                        }
                                                        Err(e) => {
                                                            self.log(&format!(
                                                                "规则加载失败: {}",
                                                                e
                                                            ));
                                                        }
                                                    }
                                                } else {
                                                    self.log(&format!(
                                                        "规则文件不存在: {}",
                                                        rules_path.display()
                                                    ));
                                                }
                                            }
                                        },
                                    );
                                });
                                ui.add_space(4.0);

                                // 显示已加载的规则
                                if let Some(rules) = &self.loaded_rules {
                                    egui::ScrollArea::vertical()
                                        .id_salt("rules_scroll")
                                        .max_height(300.0)
                                        .auto_shrink([false, false])
                                        .show(ui, |ui| {
                                            for (msg_name, rule_list) in rules {
                                                ui.label(
                                                    egui::RichText::new(format!(
                                                        "{}:",
                                                        msg_name
                                                    ))
                                                    .color(ACCENT_COLOR)
                                                    .strong(),
                                                );
                                                for rule in rule_list {
                                                    ui.label(
                                                        egui::RichText::new(format!(
                                                            "  {} = {}",
                                                            rule.output_name, rule.expression
                                                        ))
                                                        .color(LABEL_COLOR),
                                                    );
                                                }
                                                ui.add_space(4.0);
                                            }
                                        });
                                } else {
                                    ui.label(
                                        egui::RichText::new("尚未加载规则文件").color(LABEL_COLOR),
                                    );
                                }
                            });

                        ui.add_space(4.0);

                        // 4. 生成按钮
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
                                        } else {
                                            // 如果规则未加载，尝试加载
                                            let rules = if let Some(ref r) = self.loaded_rules {
                                                r.clone()
                                            } else {
                                                let rules_path = backend::get_rules_path();
                                                if !rules_path.exists() {
                                                    self.log(&format!(
                                                        "规则文件不存在: {}",
                                                        rules_path.display()
                                                    ));
                                                    return;
                                                }
                                                match backend::parse_rules_file(&rules_path) {
                                                    Ok(r) => {
                                                        self.loaded_rules = Some(r.clone());
                                                        r
                                                    }
                                                    Err(e) => {
                                                        self.log(&format!(
                                                            "规则加载失败: {}",
                                                            e
                                                        ));
                                                        return;
                                                    }
                                                }
                                            };

                                            let ulog_files =
                                                backend::find_ulog_files(&self.source_dir);
                                            if ulog_files.is_empty() {
                                                self.log("源目录中未找到 .ulg/.ulog 文件");
                                            } else {
                                                let mut ok = 0;
                                                let mut err = 0;
                                                let mut total_csv = 0;
                                                for f in &ulog_files {
                                                    match backend::export_ulog_with_rules(
                                                        f,
                                                        &self.dest_dir,
                                                        &rules,
                                                    ) {
                                                        Ok(csv_files) => {
                                                            total_csv += csv_files.len();
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
                                                    "生成完成: {} 个ulog成功, {} 个失败, 共 {} 个CSV文件",
                                                    ok, err, total_csv
                                                ));
                                            }
                                        }
                                    }
                                });
                            });

                        ui.add_space(4.0);

                        // 5. 状态信息
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

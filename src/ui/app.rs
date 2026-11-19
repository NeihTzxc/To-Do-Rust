use chrono::{Local, TimeZone};
use eframe::egui::{self, Color32, Panel, RichText, Stroke};
use std::sync::mpsc::Receiver;
use tokio::sync::mpsc::UnboundedSender;

use crate::domain::task::{Priority, Task, TaskStatus, UpdateTaskInput};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewMode {
    List,
    Kanban,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskFilter {
    All,
    Active,
    Completed,
}

#[derive(Debug, Clone)]
pub struct EditTaskState {
    pub id: String,
    pub title: String,
    pub description: String,
    pub priority: Priority,
    pub status: TaskStatus,
    pub due_date: Option<i64>,
}

#[derive(Debug)]
pub enum DbCommand {
    Refresh,
    CreateTask {
        title: String,
        description: Option<String>,
        priority: Priority,
    },
    UpdateTask(UpdateTaskInput),
    ChangeStatus {
        id: String,
        status: TaskStatus,
    },
    ToggleStatus(String),
    SoftDelete(String),
}

#[derive(Debug)]
pub enum DbEvent {
    LoadedTasks(Vec<Task>),
    Error(String),
    Notification(String),
}

enum KanbanAction {
    ChangeStatus(String, TaskStatus),
    Edit(Task),
    SoftDelete(String),
}

pub struct TaskApp {
    tx_cmd: UnboundedSender<DbCommand>,
    rx_event: Receiver<DbEvent>,

    // App state
    tasks: Vec<Task>,
    view_mode: ViewMode,
    filter: TaskFilter,
    search_query: String,

    // Form thêm task mới
    new_title: String,
    new_description: String,
    new_priority: Priority,
    show_desc_input: bool,

    // Trạng thái sửa task
    editing_task: Option<EditTaskState>,

    // Thông báo trạng thái (nội dung, is_error)
    status_message: Option<(String, bool)>,
    db_path_str: String,
}

impl TaskApp {
    pub fn new(
        tx_cmd: UnboundedSender<DbCommand>,
        rx_event: Receiver<DbEvent>,
        db_path_str: String,
    ) -> Self {
        Self {
            tx_cmd,
            rx_event,
            tasks: Vec::new(),
            view_mode: ViewMode::Kanban, // Mặc định mở Bảng Kanban trực quan
            filter: TaskFilter::All,
            search_query: String::new(),
            new_title: String::new(),
            new_description: String::new(),
            new_priority: Priority::Medium,
            show_desc_input: false,
            editing_task: None,
            status_message: Some((
                "Đã kết nối cơ sở dữ liệu SQLite thành công.".to_string(),
                false,
            )),
            db_path_str,
        }
    }

    fn submit_new_task(&mut self) {
        let title = self.new_title.trim().to_string();
        if title.is_empty() {
            self.status_message = Some(("Tiêu đề không được để trống!".to_string(), true));
            return;
        }

        let description = if self.new_description.trim().is_empty() {
            None
        } else {
            Some(self.new_description.trim().to_string())
        };

        let cmd = DbCommand::CreateTask {
            title,
            description,
            priority: self.new_priority,
        };

        if self.tx_cmd.send(cmd).is_ok() {
            self.new_title.clear();
            self.new_description.clear();
            self.show_desc_input = false;
            self.new_priority = Priority::Medium;
            self.status_message = Some(("Đang thêm công việc mới...".to_string(), false));
        }
    }

    fn save_editing_task(&mut self) {
        if let Some(edit_state) = self.editing_task.take() {
            let title = edit_state.title.trim().to_string();
            if title.is_empty() {
                self.status_message = Some(("Tiêu đề task không thể để trống".to_string(), true));
                return;
            }

            let description = if edit_state.description.trim().is_empty() {
                None
            } else {
                Some(edit_state.description.trim().to_string())
            };

            let input = UpdateTaskInput {
                id: edit_state.id,
                title,
                description,
                status: edit_state.status,
                priority: edit_state.priority,
                due_date: edit_state.due_date,
            };

            let _ = self.tx_cmd.send(DbCommand::UpdateTask(input));
            self.status_message = Some(("Đang cập nhật công việc...".to_string(), false));
        }
    }

    fn render_header(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.heading(
                    RichText::new("📝 Personal Task Manager")
                        .size(24.0)
                        .strong()
                        .color(Color32::from_rgb(46, 134, 222)),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("🔄 Làm mới").clicked() {
                        let _ = self.tx_cmd.send(DbCommand::Refresh);
                    }
                });
            });

            ui.horizontal_wrapped(|ui| {
                ui.label(
                    RichText::new("Kiến trúc Local-First (SQLite + Repository Pattern) | Sẵn sàng Cloud Sync (Supabase)")
                        .size(12.0)
                        .color(Color32::GRAY),
                );
            });

            // Thanh thống kê & trạng thái (Dùng horizontal_wrapped để không bao giờ bị tràn)
            let total_count = self.tasks.len();
            let todo_count = self.tasks.iter().filter(|t| t.status == TaskStatus::Todo).count();
            let in_progress_count = self.tasks.iter().filter(|t| t.status == TaskStatus::InProgress).count();
            let completed_count = self.tasks.iter().filter(|t| t.status.is_done()).count();
            let pending_sync_count = self.tasks.iter().filter(|t| t.sync_status == "pending").count();

            ui.add_space(4.0);
            egui::Frame::canvas(ui.style())
                .fill(ui.visuals().window_fill())
                .stroke(Stroke::new(1.0, Color32::from_gray(60)))
                .corner_radius(6.0)
                .inner_margin(8.0)
                .show(ui, |ui| {
                    ui.horizontal_wrapped(|ui| {
                        ui.label(RichText::new(format!("Tổng số: {}", total_count)).strong());
                        ui.separator();
                        ui.label(
                            RichText::new(format!("Cần làm: {}", todo_count))
                                .color(Color32::from_rgb(52, 152, 219))
                                .strong(),
                        );
                        ui.separator();
                        ui.label(
                            RichText::new(format!("Đang làm: {}", in_progress_count))
                                .color(Color32::from_rgb(255, 159, 26))
                                .strong(),
                        );
                        ui.separator();
                        ui.label(
                            RichText::new(format!("Đã xong: {}", completed_count))
                                .color(Color32::from_rgb(46, 213, 115))
                                .strong(),
                        );
                        ui.separator();
                        ui.label(
                            RichText::new(format!("Chờ đồng bộ: {}", pending_sync_count))
                                .color(Color32::from_rgb(112, 161, 255))
                                .strong(),
                        );
                    });
                });
        });
    }

    fn render_quick_add(&mut self, ui: &mut egui::Ui) {
        egui::Frame::group(ui.style())
            .corner_radius(8.0)
            .inner_margin(10.0)
            .show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.label(RichText::new("Thêm việc:").strong());

                    // Co giãn linh hoạt: tối thiểu 200px, co giãn mượt mà theo chiều rộng
                    let input_width = (ui.available_width() - 320.0).clamp(200.0, 520.0);
                    let response = ui.add(
                        egui::TextEdit::singleline(&mut self.new_title)
                            .hint_text("Nhập tên công việc mới... (Nhấn Enter để thêm)")
                            .desired_width(input_width),
                    );

                    // Bắt sự kiện nhấn phím Enter
                    if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        self.submit_new_task();
                    }

                    // Lựa chọn độ ưu tiên
                    ui.label("Ưu tiên:");
                    egui::ComboBox::from_id_salt("new_task_priority")
                        .selected_text(self.new_priority.label())
                        .width(110.0)
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                &mut self.new_priority,
                                Priority::Low,
                                Priority::Low.label(),
                            );
                            ui.selectable_value(
                                &mut self.new_priority,
                                Priority::Medium,
                                Priority::Medium.label(),
                            );
                            ui.selectable_value(
                                &mut self.new_priority,
                                Priority::High,
                                Priority::High.label(),
                            );
                        });

                    if ui
                        .button(RichText::new("➕ Thêm").strong().color(Color32::WHITE))
                        .clicked()
                    {
                        self.submit_new_task();
                    }

                    let desc_btn_label = if self.show_desc_input {
                        "Ẩn mô tả"
                    } else {
                        "+ Mô tả"
                    };
                    if ui.small_button(desc_btn_label).clicked() {
                        self.show_desc_input = !self.show_desc_input;
                    }
                });

                if self.show_desc_input {
                    ui.add_space(4.0);
                    ui.add(
                        egui::TextEdit::multiline(&mut self.new_description)
                            .hint_text("Thêm chi tiết mô tả công việc (không bắt buộc)...")
                            .desired_rows(2)
                            .desired_width(ui.available_width()),
                    );
                }
            });
    }

    fn render_filters(&mut self, ui: &mut egui::Ui) {
        ui.horizontal_wrapped(|ui| {
            // Chuyển đổi chế độ xem: Danh sách / Kanban
            ui.label(RichText::new("Chế độ:").strong());
            ui.selectable_value(&mut self.view_mode, ViewMode::Kanban, "📊 Bảng Kanban");
            ui.selectable_value(&mut self.view_mode, ViewMode::List, "📋 Danh sách");

            ui.separator();

            // Nếu ở chế độ Danh sách thì hiển thị thêm các tab lọc trạng thái
            if self.view_mode == ViewMode::List {
                ui.label(RichText::new("Lọc:").strong());
                ui.selectable_value(&mut self.filter, TaskFilter::All, "Tất cả");
                ui.selectable_value(&mut self.filter, TaskFilter::Active, "Đang làm");
                ui.selectable_value(&mut self.filter, TaskFilter::Completed, "Đã xong");
                ui.separator();
            }

            // Ô tìm kiếm co giãn linh hoạt
            let search_width = (ui.available_width() - 20.0).clamp(140.0, 240.0);
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.add(
                    egui::TextEdit::singleline(&mut self.search_query)
                        .hint_text("🔍 Tìm kiếm...")
                        .desired_width(search_width),
                );
            });
        });
    }

    // ==========================================
    // Chế độ xem: DANH SÁCH (LIST VIEW)
    // ==========================================
    fn render_list_view(&mut self, ui: &mut egui::Ui, tasks: &[Task]) {
        egui::ScrollArea::vertical()
            .id_salt("list_view_scroll")
            .auto_shrink([false, false])
            .show(ui, |ui| {
                if tasks.is_empty() {
                    ui.add_space(40.0);
                    ui.vertical_centered(|ui| {
                        ui.label(
                            RichText::new("Không có công việc nào phù hợp")
                                .size(16.0)
                                .color(Color32::GRAY),
                        );
                        ui.label(
                            RichText::new("Hãy thêm công việc mới ở khung phía trên để bắt đầu!")
                                .size(13.0)
                                .color(Color32::DARK_GRAY),
                        );
                    });
                } else {
                    for task in tasks {
                        self.render_task_item(ui, task);
                        ui.add_space(4.0);
                    }
                }
            });
    }

    fn render_task_item(&mut self, ui: &mut egui::Ui, task: &Task) {
        let is_done = task.status.is_done();
        let task_id = task.id.clone();

        egui::Frame::canvas(ui.style())
            .fill(if is_done {
                ui.visuals().faint_bg_color
            } else {
                ui.visuals().window_fill()
            })
            .stroke(Stroke::new(
                1.0,
                if is_done {
                    Color32::from_gray(50)
                } else {
                    Color32::from_gray(80)
                },
            ))
            .corner_radius(6.0)
            .inner_margin(8.0)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    // Checkbox toggle hoàn thành
                    let mut checked = is_done;
                    if ui.checkbox(&mut checked, "").changed() {
                        let _ = self.tx_cmd.send(DbCommand::ToggleStatus(task_id.clone()));
                    }

                    // Badge mức độ ưu tiên
                    let (priority_bg, priority_fg) = match task.priority {
                        Priority::High => (Color32::from_rgb(235, 77, 75), Color32::WHITE),
                        Priority::Medium => (Color32::from_rgb(240, 147, 43), Color32::WHITE),
                        Priority::Low => (Color32::from_rgb(104, 109, 224), Color32::WHITE),
                    };

                    egui::Frame::new()
                        .fill(priority_bg)
                        .corner_radius(4.0)
                        .inner_margin(egui::Vec2::new(6.0, 2.0))
                        .show(ui, |ui| {
                            ui.label(
                                RichText::new(task.priority.short_badge())
                                    .size(10.5)
                                    .strong()
                                    .color(priority_fg),
                            );
                        });

                    // Tiêu đề task (gạch ngang nếu đã xong)
                    let title_text = if is_done {
                        RichText::new(&task.title)
                            .strikethrough()
                            .color(Color32::GRAY)
                    } else {
                        RichText::new(&task.title).strong().size(14.0)
                    };

                    let title_label =
                        ui.add(egui::Label::new(title_text).wrap().sense(egui::Sense::click()));
                    if title_label.clicked() {
                        self.editing_task = Some(EditTaskState {
                            id: task.id.clone(),
                            title: task.title.clone(),
                            description: task.description.clone().unwrap_or_default(),
                            priority: task.priority,
                            status: task.status,
                            due_date: task.due_date,
                        });
                    }

                    // Bên phải: Trạng thái, thời gian tạo, nút sửa, nút xóa
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // Nút xóa mềm (soft delete)
                        if ui
                            .button(RichText::new("🗑").color(Color32::from_rgb(255, 107, 107)))
                            .on_hover_text("Xóa công việc (Soft Delete)")
                            .clicked()
                        {
                            let _ = self.tx_cmd.send(DbCommand::SoftDelete(task_id.clone()));
                        }

                        // Nút sửa
                        if ui
                            .button("✏ Sửa")
                            .on_hover_text("Chỉnh sửa tiêu đề, mô tả, ưu tiên")
                            .clicked()
                        {
                            self.editing_task = Some(EditTaskState {
                                id: task.id.clone(),
                                title: task.title.clone(),
                                description: task.description.clone().unwrap_or_default(),
                                priority: task.priority,
                                status: task.status,
                                due_date: task.due_date,
                            });
                        }

                        // Ngày tạo formatted
                        if let Some(datetime) = Local.timestamp_millis_opt(task.created_at).single()
                        {
                            ui.label(
                                RichText::new(datetime.format("%d/%m %H:%M").to_string())
                                    .size(11.0)
                                    .color(Color32::DARK_GRAY),
                            );
                        }

                        // Sync indicator badge
                        let sync_icon = if task.sync_status == "synced" {
                            RichText::new("☁").color(Color32::from_rgb(46, 213, 115))
                        } else {
                            RichText::new("⏳").color(Color32::from_rgb(255, 159, 26))
                        };
                        ui.label(sync_icon)
                            .on_hover_text(format!("Trạng thái đồng bộ: {}", task.sync_status));
                    });
                });

                // Hiển thị mô tả nếu có
                if let Some(desc) = &task.description {
                    if !desc.trim().is_empty() {
                        ui.add_space(3.0);
                        ui.horizontal(|ui| {
                            ui.add_space(24.0);
                            ui.label(
                                RichText::new(desc)
                                    .size(12.0)
                                    .italics()
                                    .color(Color32::from_gray(160)),
                            );
                        });
                    }
                }
            });
    }

    // ==========================================
    // Chế độ xem: BẢNG KANBAN (3 CỘT)
    // ==========================================
    fn render_kanban_board(&mut self, ui: &mut egui::Ui, tasks: &[Task]) {
        let todo_tasks: Vec<Task> = tasks
            .iter()
            .filter(|t| t.status == TaskStatus::Todo)
            .cloned()
            .collect();
        let in_progress_tasks: Vec<Task> = tasks
            .iter()
            .filter(|t| t.status == TaskStatus::InProgress)
            .cloned()
            .collect();
        let done_tasks: Vec<Task> = tasks
            .iter()
            .filter(|t| t.status == TaskStatus::Done)
            .cloned()
            .collect();

        let mut actions: Vec<KanbanAction> = Vec::new();

        ui.columns(3, |columns| {
            // Cột 1: Cần làm (Todo)
            Self::render_kanban_column(
                &mut columns[0],
                TaskStatus::Todo,
                "📌 CẦN LÀM",
                Color32::from_rgb(52, 152, 219),
                &todo_tasks,
                &mut actions,
            );

            // Cột 2: Đang làm (In Progress)
            Self::render_kanban_column(
                &mut columns[1],
                TaskStatus::InProgress,
                "⚡ ĐANG LÀM",
                Color32::from_rgb(243, 156, 18),
                &in_progress_tasks,
                &mut actions,
            );

            // Cột 3: Hoàn thành (Done)
            Self::render_kanban_column(
                &mut columns[2],
                TaskStatus::Done,
                "✅ HOÀN THÀNH",
                Color32::from_rgb(46, 204, 113),
                &done_tasks,
                &mut actions,
            );
        });

        // Xử lý các hành động phát sinh từ bảng Kanban (Drag & Drop, Arrow click, Sửa, Xóa)
        for action in actions {
            match action {
                KanbanAction::ChangeStatus(id, new_status) => {
                    let _ = self.tx_cmd.send(DbCommand::ChangeStatus {
                        id,
                        status: new_status,
                    });
                }
                KanbanAction::Edit(task) => {
                    self.editing_task = Some(EditTaskState {
                        id: task.id,
                        title: task.title,
                        description: task.description.unwrap_or_default(),
                        priority: task.priority,
                        status: task.status,
                        due_date: task.due_date,
                    });
                }
                KanbanAction::SoftDelete(id) => {
                    let _ = self.tx_cmd.send(DbCommand::SoftDelete(id));
                }
            }
        }
    }

    fn render_kanban_column(
        ui: &mut egui::Ui,
        col_status: TaskStatus,
        title: &str,
        accent_color: Color32,
        tasks: &[Task],
        actions: &mut Vec<KanbanAction>,
    ) {
        let col_frame = egui::Frame::canvas(ui.style())
            .fill(ui.visuals().faint_bg_color)
            .stroke(Stroke::new(1.0, Color32::from_gray(65)))
            .corner_radius(8.0)
            .inner_margin(8.0);

        // Vùng thả thẻ Kéo & Thả (Drag & Drop Drop Zone)
        let (_, dropped_id) = ui.dnd_drop_zone::<String, _>(col_frame, |ui| {
            // Tiêu đề cột + Badge số lượng
            ui.horizontal(|ui| {
                ui.heading(
                    RichText::new(title)
                        .size(13.5)
                        .strong()
                        .color(accent_color),
                );

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    egui::Frame::new()
                        .fill(accent_color.gamma_multiply(0.25))
                        .corner_radius(10.0)
                        .inner_margin(egui::Vec2::new(7.0, 2.0))
                        .show(ui, |ui| {
                            ui.label(
                                RichText::new(format!("{}", tasks.len()))
                                    .size(11.0)
                                    .strong()
                                    .color(accent_color),
                            );
                        });
                });
            });

            ui.add_space(4.0);
            ui.separator();
            ui.add_space(4.0);

            // Cuộn riêng biệt cho từng cột Kanban
            let scroll_id = format!("kanban_scroll_{}", col_status.as_str());
            egui::ScrollArea::vertical()
                .id_salt(scroll_id)
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    if tasks.is_empty() {
                        ui.add_space(35.0);
                        ui.vertical_centered(|ui| {
                            ui.label(
                                RichText::new("Chưa có việc trong mục này\n(Kéo thả thẻ vào đây)")
                                    .size(11.5)
                                    .color(Color32::from_gray(110)),
                            );
                        });
                    } else {
                        for task in tasks {
                            // Bọc từng thẻ vào dnd_drag_source để hỗ trợ kéo thả sang cột khác
                            ui.dnd_drag_source(
                                egui::Id::unique(&task.id),
                                task.id.clone(),
                                |ui| {
                                    Self::render_kanban_card(ui, task, col_status, actions);
                                },
                            );
                            ui.add_space(6.0);
                        }
                    }
                });
        });

        // Nếu có thẻ được kéo thả vào cột này, cập nhật trạng thái ngay
        if let Some(task_id) = dropped_id {
            actions.push(KanbanAction::ChangeStatus((*task_id).clone(), col_status));
        }
    }

    fn render_kanban_card(
        ui: &mut egui::Ui,
        task: &Task,
        col_status: TaskStatus,
        actions: &mut Vec<KanbanAction>,
    ) {
        let is_done = task.status.is_done();
        let card_frame = egui::Frame::canvas(ui.style())
            .fill(ui.visuals().window_fill())
            .stroke(Stroke::new(
                1.0,
                if is_done {
                    Color32::from_gray(50)
                } else {
                    Color32::from_gray(75)
                },
            ))
            .corner_radius(6.0)
            .inner_margin(8.0);

        card_frame.show(ui, |ui| {
            ui.vertical(|ui| {
                // Hàng trên: Badge mức độ ưu tiên & Cụm nút thao tác / chuyển cột nhanh
                ui.horizontal(|ui| {
                    let (priority_bg, priority_fg) = match task.priority {
                        Priority::High => (Color32::from_rgb(235, 77, 75), Color32::WHITE),
                        Priority::Medium => (Color32::from_rgb(240, 147, 43), Color32::WHITE),
                        Priority::Low => (Color32::from_rgb(104, 109, 224), Color32::WHITE),
                    };

                    egui::Frame::new()
                        .fill(priority_bg)
                        .corner_radius(4.0)
                        .inner_margin(egui::Vec2::new(5.0, 1.5))
                        .show(ui, |ui| {
                            ui.label(
                                RichText::new(task.priority.short_badge())
                                    .size(9.5)
                                    .strong()
                                    .color(priority_fg),
                            );
                        });

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // Nút xóa
                        if ui
                            .small_button(
                                RichText::new("🗑").color(Color32::from_rgb(255, 107, 107)),
                            )
                            .on_hover_text("Xóa công việc")
                            .clicked()
                        {
                            actions.push(KanbanAction::SoftDelete(task.id.clone()));
                        }

                        // Nút sửa
                        if ui
                            .small_button("✏")
                            .on_hover_text("Chỉnh sửa công việc")
                            .clicked()
                        {
                            actions.push(KanbanAction::Edit(task.clone()));
                        }

                        // Các nút mũi tên chuyển nhanh trạng thái (1-click)
                        match col_status {
                            TaskStatus::Todo => {
                                if ui
                                    .button(RichText::new("⚡ →").size(11.0))
                                    .on_hover_text("Chuyển sang 'Đang làm'")
                                    .clicked()
                                {
                                    actions.push(KanbanAction::ChangeStatus(
                                        task.id.clone(),
                                        TaskStatus::InProgress,
                                    ));
                                }
                            }
                            TaskStatus::InProgress => {
                                if ui
                                    .button(RichText::new("✅ →").size(11.0))
                                    .on_hover_text("Chuyển sang 'Hoàn thành'")
                                    .clicked()
                                {
                                    actions.push(KanbanAction::ChangeStatus(
                                        task.id.clone(),
                                        TaskStatus::Done,
                                    ));
                                }
                                if ui
                                    .button(RichText::new("← 📌").size(11.0))
                                    .on_hover_text("Quay lại 'Cần làm'")
                                    .clicked()
                                {
                                    actions.push(KanbanAction::ChangeStatus(
                                        task.id.clone(),
                                        TaskStatus::Todo,
                                    ));
                                }
                            }
                            TaskStatus::Done => {
                                if ui
                                    .button(RichText::new("← ⚡").size(11.0))
                                    .on_hover_text("Quay lại 'Đang làm'")
                                    .clicked()
                                {
                                    actions.push(KanbanAction::ChangeStatus(
                                        task.id.clone(),
                                        TaskStatus::InProgress,
                                    ));
                                }
                            }
                        }
                    });
                });

                ui.add_space(3.0);

                // Tiêu đề task (tự động wrap text, bấm vào để mở modal sửa)
                let title_text = if is_done {
                    RichText::new(&task.title)
                        .strikethrough()
                        .color(Color32::GRAY)
                        .size(13.0)
                } else {
                    RichText::new(&task.title).strong().size(13.5)
                };

                let title_label =
                    ui.add(egui::Label::new(title_text).wrap().sense(egui::Sense::click()));
                if title_label.clicked() {
                    actions.push(KanbanAction::Edit(task.clone()));
                }

                // Mô tả rút gọn (nếu có)
                if let Some(desc) = &task.description {
                    if !desc.trim().is_empty() {
                        ui.add_space(2.0);
                        let preview = if desc.chars().count() > 70 {
                            format!("{}...", desc.chars().take(67).collect::<String>())
                        } else {
                            desc.clone()
                        };
                        ui.label(
                            RichText::new(preview)
                                .size(11.5)
                                .italics()
                                .color(Color32::from_gray(160)),
                        );
                    }
                }

                ui.add_space(4.0);

                // Footer thẻ: ngày giờ tạo và icon sync
                ui.horizontal(|ui| {
                    if let Some(datetime) = Local.timestamp_millis_opt(task.created_at).single() {
                        ui.label(
                            RichText::new(datetime.format("%d/%m %H:%M").to_string())
                                .size(10.5)
                                .color(Color32::DARK_GRAY),
                        );
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let sync_icon = if task.sync_status == "synced" {
                            RichText::new("☁")
                                .size(11.0)
                                .color(Color32::from_rgb(46, 213, 115))
                        } else {
                            RichText::new("⏳")
                                .size(11.0)
                                .color(Color32::from_rgb(255, 159, 26))
                        };
                        ui.label(sync_icon)
                            .on_hover_text(format!("Trạng thái đồng bộ: {}", task.sync_status));
                    });
                });
            });
        });
    }

    fn render_edit_modal(&mut self, ctx: &egui::Context) {
        if let Some(edit_state) = &mut self.editing_task {
            let mut should_save = false;
            let mut should_cancel = false;

            egui::Window::new("✏ Chỉnh sửa công việc")
                .resizable(false)
                .collapsible(false)
                .default_width(450.0)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.vertical(|ui| {
                        ui.label("Tiêu đề:");
                        ui.add(
                            egui::TextEdit::singleline(&mut edit_state.title)
                                .desired_width(ui.available_width()),
                        );

                        ui.add_space(6.0);
                        ui.label("Mô tả chi tiết:");
                        ui.add(
                            egui::TextEdit::multiline(&mut edit_state.description)
                                .desired_rows(4)
                                .desired_width(ui.available_width()),
                        );

                        ui.add_space(6.0);
                        ui.horizontal(|ui| {
                            ui.label("Mức độ ưu tiên:");
                            egui::ComboBox::from_id_salt("edit_task_priority")
                                .selected_text(edit_state.priority.label())
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(
                                        &mut edit_state.priority,
                                        Priority::Low,
                                        Priority::Low.label(),
                                    );
                                    ui.selectable_value(
                                        &mut edit_state.priority,
                                        Priority::Medium,
                                        Priority::Medium.label(),
                                    );
                                    ui.selectable_value(
                                        &mut edit_state.priority,
                                        Priority::High,
                                        Priority::High.label(),
                                    );
                                });
                        });

                        ui.add_space(6.0);
                        ui.horizontal(|ui| {
                            ui.label("Trạng thái:");
                            egui::ComboBox::from_id_salt("edit_task_status")
                                .selected_text(edit_state.status.label())
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(
                                        &mut edit_state.status,
                                        TaskStatus::Todo,
                                        TaskStatus::Todo.label(),
                                    );
                                    ui.selectable_value(
                                        &mut edit_state.status,
                                        TaskStatus::InProgress,
                                        TaskStatus::InProgress.label(),
                                    );
                                    ui.selectable_value(
                                        &mut edit_state.status,
                                        TaskStatus::Done,
                                        TaskStatus::Done.label(),
                                    );
                                });
                        });

                        ui.add_space(12.0);
                        ui.separator();
                        ui.horizontal(|ui| {
                            if ui
                                .button(
                                    RichText::new("💾 Lưu thay đổi")
                                        .strong()
                                        .color(Color32::WHITE),
                                )
                                .clicked()
                            {
                                should_save = true;
                            }

                            if ui.button("Hủy").clicked() {
                                should_cancel = true;
                            }
                        });
                    });
                });

            if should_save {
                self.save_editing_task();
            } else if should_cancel {
                self.editing_task = None;
            }
        }
    }
}

impl eframe::App for TaskApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        // Xử lý các sự kiện bất đồng bộ gửi từ background worker
        while let Ok(event) = self.rx_event.try_recv() {
            match event {
                DbEvent::LoadedTasks(tasks) => {
                    self.tasks = tasks;
                }
                DbEvent::Error(err) => {
                    self.status_message = Some((err, true));
                }
                DbEvent::Notification(msg) => {
                    self.status_message = Some((msg, false));
                }
            }
        }

        let ctx = ui.ctx().clone();

        // Modal sửa task (nếu đang kích hoạt)
        self.render_edit_modal(&ctx);

        // Thanh trạng thái dưới đáy: thông báo bên trái, DB path bên phải (gọn gàng, không đè lấn)
        Panel::bottom("bottom_bar").show(ui, |ui| {
            ui.horizontal(|ui| {
                if let Some((msg, is_error)) = &self.status_message {
                    let color = if *is_error {
                        Color32::from_rgb(255, 71, 87)
                    } else {
                        Color32::from_rgb(46, 213, 115)
                    };
                    ui.label(RichText::new(msg).color(color).size(12.0));
                } else {
                    ui.label(RichText::new("Sẵn sàng").color(Color32::GRAY).size(12.0));
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(
                        RichText::new(format!("📁 DB: {}", self.db_path_str))
                            .size(11.0)
                            .color(Color32::DARK_GRAY),
                    );
                });
            });
        });

        // Khu vực trung tâm chính
        egui::CentralPanel::default().show(ui, |ui| {
            self.render_header(ui);

            ui.add_space(8.0);
            self.render_quick_add(ui);

            ui.add_space(8.0);
            self.render_filters(ui);

            ui.add_space(8.0);
            ui.separator();

            // Lọc danh sách task theo bộ lọc (nếu List view) và từ khóa tìm kiếm
            let query = self.search_query.trim().to_lowercase();
            let filtered_tasks: Vec<Task> = self
                .tasks
                .iter()
                .filter(|t| match self.view_mode {
                    ViewMode::List => match self.filter {
                        TaskFilter::All => true,
                        TaskFilter::Active => !t.status.is_done(),
                        TaskFilter::Completed => t.status.is_done(),
                    },
                    ViewMode::Kanban => true, // Kanban luôn hiển thị cả 3 cột
                })
                .filter(|t| {
                    if query.is_empty() {
                        true
                    } else {
                        t.title.to_lowercase().contains(&query)
                            || t.description
                                .as_ref()
                                .map(|d| d.to_lowercase().contains(&query))
                                .unwrap_or(false)
                    }
                })
                .cloned()
                .collect();

            // Hiển thị giao diện tương ứng theo chế độ xem đã chọn
            match self.view_mode {
                ViewMode::List => {
                    self.render_list_view(ui, &filtered_tasks);
                }
                ViewMode::Kanban => {
                    self.render_kanban_board(ui, &filtered_tasks);
                }
            }
        });
    }
}

/// Nạp font hệ thống Ubuntu để hỗ trợ tiếng Việt có dấu hoàn hảo
pub fn setup_vietnamese_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();

    let candidate_fonts = [
        "/usr/share/fonts/truetype/noto/NotoSans-Regular.ttf",
        "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
        "/usr/share/fonts/truetype/liberation/LiberationSans-Regular.ttf",
        "/usr/share/fonts/truetype/ubuntu/Ubuntu-R.ttf",
    ];

    let mut loaded = false;
    for path in candidate_fonts {
        if let Ok(font_data) = std::fs::read(path) {
            fonts.font_data.insert(
                "system_vietnamese".to_string(),
                std::sync::Arc::new(egui::FontData::from_owned(font_data)),
            );

            // Ưu tiên font này cho text thông thường và mono
            fonts
                .families
                .entry(egui::FontFamily::Proportional)
                .or_default()
                .insert(0, "system_vietnamese".to_string());

            fonts
                .families
                .entry(egui::FontFamily::Monospace)
                .or_default()
                .insert(0, "system_vietnamese".to_string());

            loaded = true;
            break;
        }
    }

    if loaded {
        ctx.set_fonts(fonts);
    }
}

pub mod ai;
pub mod commands;
pub mod db;
pub mod models;

use db::Database;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            let app_dir = app
                .path()
                .app_data_dir()
                .expect("Failed to get app data dir");
            let database = Database::new(app_dir).expect("Failed to initialize database");
            app.manage(database);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::list_projects,
            commands::create_project,
            commands::update_project,
            commands::delete_project,
            commands::get_project,
            commands::list_items,
            commands::create_item,
            commands::update_item,
            commands::delete_item,
            commands::list_tags,
            commands::create_tag,
            commands::delete_tag,
            commands::list_boards,
            commands::get_board,
            commands::create_board,
            commands::add_board_column,
            commands::add_board_card,
            commands::move_card,
            commands::delete_board,
            commands::delete_board_column,
            commands::delete_board_card,
            commands::list_activity,
            commands::get_dashboard_stats,
            commands::get_setting,
            commands::set_setting,
            commands::reset_app_data,
            commands::list_members,
            commands::create_member,
            commands::update_member,
            commands::delete_member,
            commands::list_tasks,
            commands::list_blocked_tasks,
            commands::create_task,
            commands::update_task,
            commands::delete_task,
            ai::chat_with_ai,
            ai::apply_ai_action,
            ai::list_ai_conversations,
            ai::get_ai_messages,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Ganymede");
}

use rusqlite::Connection;

use crate::{
    args::parser::{UserCommand, UserCreateCommand, UserDeleteCommand},
    context::Context,
    db::user::{create_user, delete_user, list_users},
};

pub fn handle_user_cmd(
    conn: &Connection,
    ctx: &Context,
    cmd: UserCommand,
) -> Result<(), String> {
    match cmd {
        UserCommand::Create(create_cmd) => handle_user_create(conn, ctx, create_cmd),
        UserCommand::List => handle_user_list(conn),
        UserCommand::Delete(delete_cmd) => handle_user_delete(conn, delete_cmd),
    }
}

fn handle_user_create(
    conn: &Connection,
    ctx: &Context,
    cmd: UserCreateCommand,
) -> Result<(), String> {
    let user_id = create_user(
        conn,
        &cmd.name,
        cmd.display_name.as_deref(),
        Some(ctx.current_user_id),
    )?;

    let display = cmd.display_name.as_deref().unwrap_or(&cmd.name);
    println!("Created user '{}' (id: {})", display, user_id);
    Ok(())
}

fn handle_user_list(conn: &Connection) -> Result<(), String> {
    let users = list_users(conn)?;

    if users.is_empty() {
        println!("No users found.");
        return Ok(());
    }

    // Print header
    println!("{:<4} {:<20} {:<30}", "ID", "Name", "Display Name");
    println!("{}", "-".repeat(54));

    for user in users {
        let display_name = user.display_name.as_deref().unwrap_or("-");
        println!("{:<4} {:<20} {:<30}", user.id, user.name, display_name);
    }

    Ok(())
}

fn handle_user_delete(conn: &Connection, cmd: UserDeleteCommand) -> Result<(), String> {
    delete_user(conn, &cmd.name)?;
    println!("Deleted user '{}'", cmd.name);
    Ok(())
}

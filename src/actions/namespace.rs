use rusqlite::Connection;

use crate::{
    args::parser::{
        NamespaceAddUserCommand, NamespaceCommand, NamespaceCreateCommand,
        NamespaceDeleteCommand, NamespaceMembersCommand, NamespaceRemoveUserCommand,
        NamespaceSwitchCommand,
    },
    context::Context,
    db::namespace::{
        add_user_to_namespace, create_namespace, delete_namespace,
        list_namespace_members, list_namespaces, remove_user_from_namespace,
    },
};

pub fn handle_namespace_cmd(
    conn: &Connection,
    ctx: &Context,
    cmd: NamespaceCommand,
) -> Result<(), String> {
    match cmd {
        NamespaceCommand::Create(create_cmd) => handle_ns_create(conn, ctx, create_cmd),
        NamespaceCommand::List => handle_ns_list(conn),
        NamespaceCommand::Delete(delete_cmd) => handle_ns_delete(conn, delete_cmd),
        NamespaceCommand::Switch(switch_cmd) => handle_ns_switch(switch_cmd),
        NamespaceCommand::AddUser(add_cmd) => handle_ns_add_user(conn, add_cmd),
        NamespaceCommand::RemoveUser(remove_cmd) => handle_ns_remove_user(conn, remove_cmd),
        NamespaceCommand::Members(members_cmd) => handle_ns_members(conn, ctx, members_cmd),
    }
}

fn handle_ns_create(
    conn: &Connection,
    ctx: &Context,
    cmd: NamespaceCreateCommand,
) -> Result<(), String> {
    let ns_id = create_namespace(
        conn,
        &cmd.name,
        cmd.description.as_deref(),
        ctx.current_user_id,
    )?;

    println!("Created namespace '{}' (id: {})", cmd.name, ns_id);
    Ok(())
}

fn handle_ns_list(conn: &Connection) -> Result<(), String> {
    let namespaces = list_namespaces(conn)?;

    if namespaces.is_empty() {
        println!("No namespaces found.");
        return Ok(());
    }

    // Print header
    println!("{:<4} {:<20} {:<40}", "ID", "Name", "Description");
    println!("{}", "-".repeat(64));

    for ns in namespaces {
        let description = ns.description.as_deref().unwrap_or("-");
        println!("{:<4} {:<20} {:<40}", ns.id, ns.name, description);
    }

    Ok(())
}

fn handle_ns_delete(conn: &Connection, cmd: NamespaceDeleteCommand) -> Result<(), String> {
    delete_namespace(conn, &cmd.name)?;
    println!("Deleted namespace '{}'", cmd.name);
    Ok(())
}

fn handle_ns_switch(cmd: NamespaceSwitchCommand) -> Result<(), String> {
    // For now, just tell the user to use the --ns flag or set CTM_NAMESPACE env
    // Full config file support would require modifying config module
    println!(
        "To switch to namespace '{}', either:",
        cmd.name
    );
    println!("  - Use: ctm --ns {} <command>", cmd.name);
    println!("  - Set: export CTM_NAMESPACE={}", cmd.name);
    println!("\nPersistent config support coming in a future update.");
    Ok(())
}

fn handle_ns_add_user(conn: &Connection, cmd: NamespaceAddUserCommand) -> Result<(), String> {
    add_user_to_namespace(conn, &cmd.namespace, &cmd.user, &cmd.role)?;
    println!(
        "Added user '{}' to namespace '{}' with role '{}'",
        cmd.user, cmd.namespace, cmd.role
    );
    Ok(())
}

fn handle_ns_remove_user(
    conn: &Connection,
    cmd: NamespaceRemoveUserCommand,
) -> Result<(), String> {
    remove_user_from_namespace(conn, &cmd.namespace, &cmd.user)?;
    println!(
        "Removed user '{}' from namespace '{}'",
        cmd.user, cmd.namespace
    );
    Ok(())
}

fn handle_ns_members(
    conn: &Connection,
    ctx: &Context,
    cmd: NamespaceMembersCommand,
) -> Result<(), String> {
    let namespace_name = cmd.namespace.as_deref().unwrap_or(&ctx.current_namespace_name);
    let members = list_namespace_members(conn, namespace_name)?;

    if members.is_empty() {
        println!("No members in namespace '{}'.", namespace_name);
        return Ok(());
    }

    println!("Members of namespace '{}':", namespace_name);
    println!("{:<20} {:<10}", "User", "Role");
    println!("{}", "-".repeat(30));

    for member in members {
        let user_name = member.user_name.as_deref().unwrap_or("?");
        println!("{:<20} {:<10}", user_name, member.role);
    }

    Ok(())
}

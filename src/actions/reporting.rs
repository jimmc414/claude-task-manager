use std::collections::HashMap;

use chrono::Local;
use rusqlite::Connection;
use serde_json::json;

use crate::{
    args::estimate::format_estimate,
    args::parser::{StatsCommand, TeamCommand, WorkloadCommand},
    context::Context,
    db::{
        crud::query_items,
        item::{ItemQuery, TASK},
        user::{get_user_by_name, list_users, User},
    },
};

/// Team member stats for reporting
#[derive(Debug)]
struct UserStats {
    user_id: Option<i64>,
    user_name: String,
    display_name: String,
    open_count: i64,
    done_count: i64,
}

/// Workload stats for reporting
#[derive(Debug)]
struct WorkloadStats {
    user_id: i64,
    user_name: String,
    display_name: String,
    task_count: i64,
    total_minutes: i64,
}

/// Handles the team command - shows task distribution by user
pub fn handle_team(conn: &Connection, _ctx: &Context, cmd: &TeamCommand) -> Result<(), String> {
    let users = list_users(conn)?;

    // Query all tasks (open and done)
    let all_tasks = query_items(conn, &ItemQuery::new().with_action(TASK))
        .map_err(|e| format!("Failed to query tasks: {:?}", e))?;

    // Group by assignee
    let mut stats: HashMap<Option<i64>, UserStats> = HashMap::new();

    // Initialize with all users
    for user in &users {
        let display = user.display_name.as_ref().unwrap_or(&user.name);
        stats.insert(Some(user.id), UserStats {
            user_id: Some(user.id),
            user_name: user.name.clone(),
            display_name: display.clone(),
            open_count: 0,
            done_count: 0,
        });
    }

    // Add unassigned bucket
    stats.insert(None, UserStats {
        user_id: None,
        user_name: "unassigned".to_string(),
        display_name: "unassigned".to_string(),
        open_count: 0,
        done_count: 0,
    });

    // Count tasks per user
    for task in &all_tasks {
        let entry = stats.entry(task.assignee_id).or_insert_with(|| {
            // Unknown user (assigned but deleted)
            UserStats {
                user_id: task.assignee_id,
                user_name: format!("user_{}", task.assignee_id.unwrap_or(0)),
                display_name: format!("Unknown ({})", task.assignee_id.unwrap_or(0)),
                open_count: 0,
                done_count: 0,
            }
        });

        if task.status == 1 {
            entry.done_count += 1;
        } else if task.status == 0 || task.status == 4 || task.status == 6 {
            // ongoing, suspended, pending
            entry.open_count += 1;
        }
    }

    // Sort by user name, with unassigned last
    let mut sorted_stats: Vec<_> = stats.into_values().collect();
    sorted_stats.sort_by(|a, b| {
        match (&a.user_id, &b.user_id) {
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (Some(_), None) => std::cmp::Ordering::Less,
            _ => a.user_name.cmp(&b.user_name),
        }
    });

    // Filter out users with no tasks
    let active_stats: Vec<_> = sorted_stats.into_iter()
        .filter(|s| s.open_count > 0 || s.done_count > 0)
        .collect();

    if cmd.json {
        print_team_json(&active_stats);
    } else if cmd.md {
        print_team_markdown(&active_stats);
    } else {
        print_team_text(&active_stats);
    }

    Ok(())
}

fn print_team_text(stats: &[UserStats]) {
    let total_open: i64 = stats.iter().map(|s| s.open_count).sum();
    let total_done: i64 = stats.iter().map(|s| s.done_count).sum();
    let total: i64 = total_open + total_done;

    println!();
    println!("\x1b[1mTeam Overview\x1b[0m");
    println!("{}", "━".repeat(50));
    println!("{:<20} {:>8} {:>8} {:>8}", "User", "Open", "Done", "Total");
    println!("{}", "━".repeat(50));

    for stat in stats {
        let total = stat.open_count + stat.done_count;
        let name = if stat.display_name.len() > 18 {
            format!("{}...", &stat.display_name[..15])
        } else {
            stat.display_name.clone()
        };

        if stat.user_id.is_none() {
            println!("\x1b[33m{:<20}\x1b[0m {:>8} {:>8} {:>8}", name, stat.open_count, "-", total);
        } else {
            println!("{:<20} {:>8} {:>8} {:>8}", name, stat.open_count, stat.done_count, total);
        }
    }

    println!("{}", "━".repeat(50));
    println!("{:<20} {:>8} {:>8} {:>8}", "Total", total_open, total_done, total);
    println!();
}

fn print_team_json(stats: &[UserStats]) {
    let total_open: i64 = stats.iter().map(|s| s.open_count).sum();
    let total_done: i64 = stats.iter().map(|s| s.done_count).sum();

    let team: Vec<_> = stats.iter().map(|s| {
        json!({
            "user": s.user_name,
            "display_name": s.display_name,
            "open": s.open_count,
            "done": s.done_count,
            "total": s.open_count + s.done_count
        })
    }).collect();

    let output = json!({
        "team": team,
        "totals": {
            "open": total_open,
            "done": total_done,
            "total": total_open + total_done
        }
    });

    println!("{}", serde_json::to_string_pretty(&output).unwrap());
}

fn print_team_markdown(stats: &[UserStats]) {
    let total_open: i64 = stats.iter().map(|s| s.open_count).sum();
    let total_done: i64 = stats.iter().map(|s| s.done_count).sum();
    let total: i64 = total_open + total_done;

    println!("# Team Overview\n");
    println!("| User | Open | Done | Total |");
    println!("|------|------|------|-------|");

    for stat in stats {
        let total = stat.open_count + stat.done_count;
        println!("| {} | {} | {} | {} |", stat.display_name, stat.open_count, stat.done_count, total);
    }

    println!("| **Total** | **{}** | **{}** | **{}** |", total_open, total_done, total);
}

/// Handles the workload command - shows estimated hours per user
pub fn handle_workload(conn: &Connection, _ctx: &Context, cmd: &WorkloadCommand) -> Result<(), String> {
    // Get users to filter by
    let users: Vec<User> = if let Some(ref username) = cmd.user {
        let user = get_user_by_name(conn, username)?
            .ok_or_else(|| format!("User '{}' not found", username))?;
        vec![user]
    } else {
        list_users(conn)?
    };

    // Query open tasks only
    let open_tasks = query_items(conn, &ItemQuery::new()
        .with_action(TASK)
        .with_statuses(vec![0, 4, 6]))  // ongoing, suspended, pending
        .map_err(|e| format!("Failed to query tasks: {:?}", e))?;

    // Calculate workload per user
    let mut workload_stats: Vec<WorkloadStats> = Vec::new();

    for user in &users {
        let user_tasks: Vec<_> = open_tasks.iter()
            .filter(|t| t.assignee_id == Some(user.id))
            .collect();

        let total_minutes: i64 = user_tasks.iter()
            .filter_map(|t| t.estimate_minutes)
            .sum();

        if user_tasks.is_empty() && cmd.user.is_none() {
            continue; // Skip users with no tasks unless specifically requested
        }

        workload_stats.push(WorkloadStats {
            user_id: user.id,
            user_name: user.name.clone(),
            display_name: user.display_name.as_ref().unwrap_or(&user.name).clone(),
            task_count: user_tasks.len() as i64,
            total_minutes,
        });
    }

    // Sort by workload descending
    workload_stats.sort_by(|a, b| b.total_minutes.cmp(&a.total_minutes));

    if cmd.json {
        print_workload_json(&workload_stats);
    } else if cmd.md {
        print_workload_markdown(&workload_stats);
    } else {
        print_workload_text(&workload_stats);
    }

    Ok(())
}

fn print_workload_text(stats: &[WorkloadStats]) {
    let total_tasks: i64 = stats.iter().map(|s| s.task_count).sum();
    let total_minutes: i64 = stats.iter().map(|s| s.total_minutes).sum();

    println!();
    println!("\x1b[1mWorkload Summary\x1b[0m");
    println!("{}", "━".repeat(50));
    println!("{:<20} {:>10} {:>15}", "User", "Tasks", "Estimated");
    println!("{}", "━".repeat(50));

    for stat in stats {
        let name = if stat.display_name.len() > 18 {
            format!("{}...", &stat.display_name[..15])
        } else {
            stat.display_name.clone()
        };

        let estimate = format_estimate(Some(stat.total_minutes));
        println!("{:<20} {:>10} {:>15}", name, stat.task_count, estimate);
    }

    println!("{}", "━".repeat(50));
    println!("{:<20} {:>10} {:>15}", "Total", total_tasks, format_estimate(Some(total_minutes)));
    println!();
}

fn print_workload_json(stats: &[WorkloadStats]) {
    let total_tasks: i64 = stats.iter().map(|s| s.task_count).sum();
    let total_minutes: i64 = stats.iter().map(|s| s.total_minutes).sum();

    let workload: Vec<_> = stats.iter().map(|s| {
        json!({
            "user": s.user_name,
            "display_name": s.display_name,
            "tasks": s.task_count,
            "estimated_minutes": s.total_minutes,
            "estimated_formatted": format_estimate(Some(s.total_minutes))
        })
    }).collect();

    let output = json!({
        "workload": workload,
        "totals": {
            "tasks": total_tasks,
            "estimated_minutes": total_minutes,
            "estimated_formatted": format_estimate(Some(total_minutes))
        }
    });

    println!("{}", serde_json::to_string_pretty(&output).unwrap());
}

fn print_workload_markdown(stats: &[WorkloadStats]) {
    let total_tasks: i64 = stats.iter().map(|s| s.task_count).sum();
    let total_minutes: i64 = stats.iter().map(|s| s.total_minutes).sum();

    println!("# Workload Summary\n");
    println!("| User | Tasks | Estimated |");
    println!("|------|-------|-----------|");

    for stat in stats {
        let estimate = format_estimate(Some(stat.total_minutes));
        println!("| {} | {} | {} |", stat.display_name, stat.task_count, estimate);
    }

    println!("| **Total** | **{}** | **{}** |", total_tasks, format_estimate(Some(total_minutes)));
}

/// Handles the stats command - shows completion rates and overdue analysis
pub fn handle_stats(conn: &Connection, _ctx: &Context, cmd: &StatsCommand) -> Result<(), String> {
    let now = Local::now().timestamp();
    let cutoff = now - (cmd.days * 86400);

    // Query all tasks
    let all_tasks = query_items(conn, &ItemQuery::new().with_action(TASK))
        .map_err(|e| format!("Failed to query tasks: {:?}", e))?;

    // Calculate stats
    let created_in_period: usize = all_tasks.iter()
        .filter(|t| t.create_time >= cutoff)
        .count();

    let completed_in_period: usize = all_tasks.iter()
        .filter(|t| t.status == 1 && t.create_time >= cutoff)
        .count();

    let completion_rate = if created_in_period > 0 {
        (completed_in_period as f64 / created_in_period as f64 * 100.0) as i64
    } else {
        0
    };

    // Overdue: open tasks with target_time < now
    let overdue: usize = all_tasks.iter()
        .filter(|t| {
            (t.status == 0 || t.status == 4 || t.status == 6) &&
            t.target_time.map(|tt| tt < now).unwrap_or(false)
        })
        .count();

    // High priority open tasks
    let high_priority: usize = all_tasks.iter()
        .filter(|t| {
            (t.status == 0 || t.status == 4 || t.status == 6) &&
            t.priority == Some(0)
        })
        .count();

    // Status breakdown
    let ongoing: usize = all_tasks.iter().filter(|t| t.status == 0).count();
    let pending: usize = all_tasks.iter().filter(|t| t.status == 6).count();
    let suspended: usize = all_tasks.iter().filter(|t| t.status == 4).count();
    let done: usize = all_tasks.iter().filter(|t| t.status == 1).count();
    let cancelled: usize = all_tasks.iter().filter(|t| t.status == 2).count();

    if cmd.json {
        print_stats_json(cmd.days, created_in_period, completed_in_period, completion_rate,
                         overdue, high_priority, ongoing, pending, suspended, done, cancelled);
    } else if cmd.md {
        print_stats_markdown(cmd.days, created_in_period, completed_in_period, completion_rate,
                            overdue, high_priority, ongoing, pending, suspended, done, cancelled);
    } else {
        print_stats_text(cmd.days, created_in_period, completed_in_period, completion_rate,
                        overdue, high_priority, ongoing, pending, suspended, done, cancelled);
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn print_stats_text(
    days: i64,
    created: usize,
    completed: usize,
    completion_rate: i64,
    overdue: usize,
    high_priority: usize,
    ongoing: usize,
    pending: usize,
    suspended: usize,
    done: usize,
    cancelled: usize,
) {
    println!();
    println!("\x1b[1mTask Statistics (last {} days)\x1b[0m", days);
    println!("{}", "━".repeat(40));
    println!("Created:        {}", created);
    println!("Completed:      {}", completed);
    println!("Completion:     {}%", completion_rate);
    println!("{}", "━".repeat(40));

    if overdue > 0 {
        println!("Overdue:        \x1b[91m{}\x1b[0m", overdue);
    } else {
        println!("Overdue:        {}", overdue);
    }

    if high_priority > 0 {
        println!("High Priority:  \x1b[91m{}\x1b[0m", high_priority);
    } else {
        println!("High Priority:  {}", high_priority);
    }

    println!("{}", "━".repeat(40));
    println!("By Status:");
    println!("  ongoing       {}", ongoing);
    println!("  pending       {}", pending);
    println!("  suspended     {}", suspended);
    println!("  done          {}", done);
    println!("  cancelled     {}", cancelled);
    println!();
}

#[allow(clippy::too_many_arguments)]
fn print_stats_json(
    days: i64,
    created: usize,
    completed: usize,
    completion_rate: i64,
    overdue: usize,
    high_priority: usize,
    ongoing: usize,
    pending: usize,
    suspended: usize,
    done: usize,
    cancelled: usize,
) {
    let output = json!({
        "period_days": days,
        "created": created,
        "completed": completed,
        "completion_rate": completion_rate,
        "overdue": overdue,
        "high_priority": high_priority,
        "by_status": {
            "ongoing": ongoing,
            "pending": pending,
            "suspended": suspended,
            "done": done,
            "cancelled": cancelled
        }
    });

    println!("{}", serde_json::to_string_pretty(&output).unwrap());
}

#[allow(clippy::too_many_arguments)]
fn print_stats_markdown(
    days: i64,
    created: usize,
    completed: usize,
    completion_rate: i64,
    overdue: usize,
    high_priority: usize,
    ongoing: usize,
    pending: usize,
    suspended: usize,
    done: usize,
    cancelled: usize,
) {
    println!("# Task Statistics (last {} days)\n", days);
    println!("| Metric | Value |");
    println!("|--------|-------|");
    println!("| Created | {} |", created);
    println!("| Completed | {} |", completed);
    println!("| Completion Rate | {}% |", completion_rate);
    println!("| Overdue | {} |", overdue);
    println!("| High Priority | {} |", high_priority);
    println!("\n## By Status\n");
    println!("| Status | Count |");
    println!("|--------|-------|");
    println!("| ongoing | {} |", ongoing);
    println!("| pending | {} |", pending);
    println!("| suspended | {} |", suspended);
    println!("| done | {} |", done);
    println!("| cancelled | {} |", cancelled);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::{get_test_conn, insert_task};

    #[test]
    fn test_handle_team() {
        let (conn, _temp_file) = get_test_conn();
        let ctx = Context::default_from_db(&conn).unwrap();

        // Create some tasks
        insert_task(&conn, "work", "Task 1", "today");
        insert_task(&conn, "work", "Task 2", "tomorrow");

        let cmd = TeamCommand { json: false, md: false };
        let result = handle_team(&conn, &ctx, &cmd);
        assert!(result.is_ok());
    }

    #[test]
    fn test_handle_team_json() {
        let (conn, _temp_file) = get_test_conn();
        let ctx = Context::default_from_db(&conn).unwrap();

        insert_task(&conn, "work", "Task 1", "today");

        let cmd = TeamCommand { json: true, md: false };
        let result = handle_team(&conn, &ctx, &cmd);
        assert!(result.is_ok());
    }

    #[test]
    fn test_handle_team_md() {
        let (conn, _temp_file) = get_test_conn();
        let ctx = Context::default_from_db(&conn).unwrap();

        insert_task(&conn, "work", "Task 1", "today");

        let cmd = TeamCommand { json: false, md: true };
        let result = handle_team(&conn, &ctx, &cmd);
        assert!(result.is_ok());
    }

    #[test]
    fn test_handle_workload() {
        let (conn, _temp_file) = get_test_conn();
        let ctx = Context::default_from_db(&conn).unwrap();

        insert_task(&conn, "work", "Task 1", "today");

        let cmd = WorkloadCommand { user: None, json: false, md: false };
        let result = handle_workload(&conn, &ctx, &cmd);
        assert!(result.is_ok());
    }

    #[test]
    fn test_handle_workload_user_filter() {
        let (conn, _temp_file) = get_test_conn();
        let ctx = Context::default_from_db(&conn).unwrap();

        insert_task(&conn, "work", "Task 1", "today");

        // Use the actual default user name from the context
        let cmd = WorkloadCommand { user: Some(ctx.current_user_name.clone()), json: false, md: false };
        let result = handle_workload(&conn, &ctx, &cmd);
        assert!(result.is_ok());
    }

    #[test]
    fn test_handle_workload_user_not_found() {
        let (conn, _temp_file) = get_test_conn();
        let ctx = Context::default_from_db(&conn).unwrap();

        let cmd = WorkloadCommand { user: Some("nonexistent".to_string()), json: false, md: false };
        let result = handle_workload(&conn, &ctx, &cmd);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));
    }

    #[test]
    fn test_handle_stats() {
        let (conn, _temp_file) = get_test_conn();
        let ctx = Context::default_from_db(&conn).unwrap();

        insert_task(&conn, "work", "Task 1", "today");
        insert_task(&conn, "work", "Task 2", "tomorrow");

        let cmd = StatsCommand { days: 30, json: false, md: false };
        let result = handle_stats(&conn, &ctx, &cmd);
        assert!(result.is_ok());
    }

    #[test]
    fn test_handle_stats_json() {
        let (conn, _temp_file) = get_test_conn();
        let ctx = Context::default_from_db(&conn).unwrap();

        insert_task(&conn, "work", "Task 1", "today");

        let cmd = StatsCommand { days: 30, json: true, md: false };
        let result = handle_stats(&conn, &ctx, &cmd);
        assert!(result.is_ok());
    }

    #[test]
    fn test_handle_stats_custom_days() {
        let (conn, _temp_file) = get_test_conn();
        let ctx = Context::default_from_db(&conn).unwrap();

        let cmd = StatsCommand { days: 7, json: false, md: false };
        let result = handle_stats(&conn, &ctx, &cmd);
        assert!(result.is_ok());
    }
}

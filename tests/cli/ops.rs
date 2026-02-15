use crate::helpers::db::setup_test_db_shared;
use anyhow::Result;
use judo::app::App;
use judo::cli::ops::{add_item, add_list, delete_item, delete_list, toggle_done_item};
use judo::db::config::{Config, DBConfig};
use judo::db::models::{NewTodoList, TodoItem, TodoList};

/// Build a test App backed by a named shared in-memory database.
///
/// Because the connection string uses `cache=shared`, the pools that
/// CLI ops create internally (via `get_db_pool`) will hit the **same**
/// in-memory database as the pool stored on the returned `App`.
async fn setup_test_app() -> Result<App> {
    let (pool, connection_str) = setup_test_db_shared().await?;

    let test_db_config = DBConfig {
        name: "test_db".to_string(),
        connection_str,
    };

    let config = Config {
        default: "test_db".to_string(),
        dbs: vec![test_db_config.clone()],
        colours: Default::default(),
    };

    Ok(App {
        config,
        current_db_config: test_db_config,
        current_screen: judo::app::state::CurrentScreen::Main,
        pool,
        lists_component: judo::ui::components::ListsComponent::new(),
        input_state: judo::ui::components::InputState::new(),
        selected_db_index: 0,
        exit: false,
    })
}

// ===== List Operations Tests =====

#[tokio::test]
async fn test_add_list_default_db() -> Result<()> {
    let app = setup_test_app().await?;

    add_list(&app, "Shopping List".to_string(), &None).await?;

    let lists = TodoList::get_all(&app.pool).await?;
    assert_eq!(lists.len(), 1);
    assert_eq!(lists[0].name, "Shopping List");

    Ok(())
}

#[tokio::test]
async fn test_add_multiple_lists() -> Result<()> {
    let app = setup_test_app().await?;

    add_list(&app, "List 1".to_string(), &None).await?;
    add_list(&app, "List 2".to_string(), &None).await?;
    add_list(&app, "List 3".to_string(), &None).await?;

    let lists = TodoList::get_all(&app.pool).await?;
    assert_eq!(lists.len(), 3);

    let names: Vec<String> = lists.iter().map(|l| l.name.clone()).collect();
    assert!(names.contains(&"List 1".to_string()));
    assert!(names.contains(&"List 2".to_string()));
    assert!(names.contains(&"List 3".to_string()));

    Ok(())
}

#[tokio::test]
async fn test_add_list_with_empty_name() -> Result<()> {
    let app = setup_test_app().await?;

    add_list(&app, "".to_string(), &None).await?;

    let lists = TodoList::get_all(&app.pool).await?;
    assert_eq!(lists.len(), 1);
    assert_eq!(lists[0].name, "");

    Ok(())
}

#[tokio::test]
async fn test_add_list_with_special_characters() -> Result<()> {
    let app = setup_test_app().await?;

    let special_name = "Special List! @#$% & *()";
    add_list(&app, special_name.to_string(), &None).await?;

    let lists = TodoList::get_all(&app.pool).await?;
    assert_eq!(lists.len(), 1);
    assert_eq!(lists[0].name, special_name);

    Ok(())
}

#[tokio::test]
async fn test_add_list_with_unicode() -> Result<()> {
    let app = setup_test_app().await?;

    let unicode_name = "🚀 Rocket List 测试";
    add_list(&app, unicode_name.to_string(), &None).await?;

    let lists = TodoList::get_all(&app.pool).await?;
    assert_eq!(lists.len(), 1);
    assert_eq!(lists[0].name, unicode_name);

    Ok(())
}

#[tokio::test]
async fn test_delete_list_by_name() -> Result<()> {
    let app = setup_test_app().await?;

    add_list(&app, "To Delete".to_string(), &None).await?;
    assert_eq!(TodoList::get_all(&app.pool).await?.len(), 1);

    delete_list(&app, Some("To Delete".to_string()), None, &None).await?;

    assert_eq!(TodoList::get_all(&app.pool).await?.len(), 0);

    Ok(())
}

#[tokio::test]
async fn test_delete_list_by_id() -> Result<()> {
    let app = setup_test_app().await?;

    let created = TodoList::create(
        &app.pool,
        NewTodoList {
            name: "To Delete".to_string(),
        },
    )
    .await?;

    delete_list(&app, None, Some(created.id), &None).await?;

    assert_eq!(TodoList::get_all(&app.pool).await?.len(), 0);

    Ok(())
}

#[tokio::test]
async fn test_delete_list_keeps_others() -> Result<()> {
    let app = setup_test_app().await?;

    add_list(&app, "Keep 1".to_string(), &None).await?;
    add_list(&app, "Delete Me".to_string(), &None).await?;
    add_list(&app, "Keep 2".to_string(), &None).await?;

    delete_list(&app, Some("Delete Me".to_string()), None, &None).await?;

    let lists = TodoList::get_all(&app.pool).await?;
    assert_eq!(lists.len(), 2);

    let names: Vec<String> = lists.iter().map(|l| l.name.clone()).collect();
    assert!(names.contains(&"Keep 1".to_string()));
    assert!(names.contains(&"Keep 2".to_string()));
    assert!(!names.contains(&"Delete Me".to_string()));

    Ok(())
}

// ===== Item Operations Tests =====

#[tokio::test]
async fn test_add_item_to_list_by_name() -> Result<()> {
    let app = setup_test_app().await?;

    add_list(&app, "Shopping".to_string(), &None).await?;
    add_item(
        &app,
        "Buy milk".to_string(),
        &None,
        None,
        Some("Shopping".to_string()),
    )
    .await?;

    let lists = TodoList::get_all(&app.pool).await?;
    let items = lists[0].get_all_items(&app.pool).await?;
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].name, "Buy milk");
    assert!(!items[0].is_done);

    Ok(())
}

#[tokio::test]
async fn test_add_item_to_list_by_id() -> Result<()> {
    let app = setup_test_app().await?;

    let created = TodoList::create(
        &app.pool,
        NewTodoList {
            name: "Tasks".to_string(),
        },
    )
    .await?;

    add_item(
        &app,
        "Complete project".to_string(),
        &None,
        Some(created.id),
        None,
    )
    .await?;

    let items = TodoItem::get_by_list_id(&app.pool, created.id).await?;
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].name, "Complete project");

    Ok(())
}

#[tokio::test]
async fn test_add_multiple_items_to_list() -> Result<()> {
    let app = setup_test_app().await?;

    add_list(&app, "Work".to_string(), &None).await?;

    for name in ["Item 1", "Item 2", "Item 3"] {
        add_item(
            &app,
            name.to_string(),
            &None,
            None,
            Some("Work".to_string()),
        )
        .await?;
    }

    let lists = TodoList::get_all(&app.pool).await?;
    let items = lists[0].get_all_items(&app.pool).await?;
    assert_eq!(items.len(), 3);

    let names: Vec<String> = items.iter().map(|i| i.name.clone()).collect();
    assert!(names.contains(&"Item 1".to_string()));
    assert!(names.contains(&"Item 2".to_string()));
    assert!(names.contains(&"Item 3".to_string()));

    Ok(())
}

#[tokio::test]
async fn test_add_item_with_special_characters() -> Result<()> {
    let app = setup_test_app().await?;

    add_list(&app, "Test".to_string(), &None).await?;

    let special_name = "Special! @#$% & *() item";
    add_item(
        &app,
        special_name.to_string(),
        &None,
        None,
        Some("Test".to_string()),
    )
    .await?;

    let lists = TodoList::get_all(&app.pool).await?;
    let items = lists[0].get_all_items(&app.pool).await?;
    assert_eq!(items[0].name, special_name);

    Ok(())
}

#[tokio::test]
async fn test_add_item_with_unicode() -> Result<()> {
    let app = setup_test_app().await?;

    add_list(&app, "Test".to_string(), &None).await?;

    let unicode_name = "🎉 Party 测试 item";
    add_item(
        &app,
        unicode_name.to_string(),
        &None,
        None,
        Some("Test".to_string()),
    )
    .await?;

    let lists = TodoList::get_all(&app.pool).await?;
    let items = lists[0].get_all_items(&app.pool).await?;
    assert_eq!(items[0].name, unicode_name);

    Ok(())
}

#[tokio::test]
async fn test_delete_item_by_id() -> Result<()> {
    let app = setup_test_app().await?;

    add_list(&app, "Test".to_string(), &None).await?;
    add_item(
        &app,
        "To Delete".to_string(),
        &None,
        None,
        Some("Test".to_string()),
    )
    .await?;

    let lists = TodoList::get_all(&app.pool).await?;
    let items = lists[0].get_all_items(&app.pool).await?;
    assert_eq!(items.len(), 1);

    delete_item(&app, items[0].id, &None).await?;

    assert_eq!(lists[0].get_all_items(&app.pool).await?.len(), 0);

    Ok(())
}

#[tokio::test]
async fn test_delete_item_keeps_others() -> Result<()> {
    let app = setup_test_app().await?;

    add_list(&app, "Test".to_string(), &None).await?;
    for name in ["Item 1", "Item 2", "Item 3"] {
        add_item(
            &app,
            name.to_string(),
            &None,
            None,
            Some("Test".to_string()),
        )
        .await?;
    }

    let lists = TodoList::get_all(&app.pool).await?;
    let items = lists[0].get_all_items(&app.pool).await?;
    let id_to_delete = items.iter().find(|i| i.name == "Item 2").unwrap().id;

    delete_item(&app, id_to_delete, &None).await?;

    let remaining = lists[0].get_all_items(&app.pool).await?;
    assert_eq!(remaining.len(), 2);

    let names: Vec<String> = remaining.iter().map(|i| i.name.clone()).collect();
    assert!(names.contains(&"Item 1".to_string()));
    assert!(names.contains(&"Item 3".to_string()));
    assert!(!names.contains(&"Item 2".to_string()));

    Ok(())
}

#[tokio::test]
async fn test_toggle_done_item() -> Result<()> {
    let app = setup_test_app().await?;

    add_list(&app, "Test".to_string(), &None).await?;
    add_item(
        &app,
        "Toggle Me".to_string(),
        &None,
        None,
        Some("Test".to_string()),
    )
    .await?;

    let lists = TodoList::get_all(&app.pool).await?;
    let items = lists[0].get_all_items(&app.pool).await?;
    let item_id = items[0].id;
    assert!(!items[0].is_done);

    // Toggle to done
    toggle_done_item(&app, item_id, &None).await?;
    let item = TodoItem::get_by_id(&app.pool, item_id).await?.unwrap();
    assert!(item.is_done);

    // Toggle back to not done
    toggle_done_item(&app, item_id, &None).await?;
    let item = TodoItem::get_by_id(&app.pool, item_id).await?.unwrap();
    assert!(!item.is_done);

    Ok(())
}

#[tokio::test]
async fn test_toggle_done_multiple_times() -> Result<()> {
    let app = setup_test_app().await?;

    add_list(&app, "Test".to_string(), &None).await?;
    add_item(
        &app,
        "Toggle Test".to_string(),
        &None,
        None,
        Some("Test".to_string()),
    )
    .await?;

    let lists = TodoList::get_all(&app.pool).await?;
    let items = lists[0].get_all_items(&app.pool).await?;
    let item_id = items[0].id;

    for i in 0..10 {
        toggle_done_item(&app, item_id, &None).await?;
        let item = TodoItem::get_by_id(&app.pool, item_id).await?.unwrap();
        let expected_done = (i + 1) % 2 == 1;
        assert_eq!(item.is_done, expected_done);
    }

    Ok(())
}

// ===== Edge Cases =====

#[tokio::test]
async fn test_add_item_with_empty_name() -> Result<()> {
    let app = setup_test_app().await?;

    add_list(&app, "Test".to_string(), &None).await?;
    add_item(&app, "".to_string(), &None, None, Some("Test".to_string())).await?;

    let lists = TodoList::get_all(&app.pool).await?;
    let items = lists[0].get_all_items(&app.pool).await?;
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].name, "");

    Ok(())
}

#[tokio::test]
async fn test_add_item_with_very_long_name() -> Result<()> {
    let app = setup_test_app().await?;

    add_list(&app, "Test".to_string(), &None).await?;

    let long_name = "A".repeat(1000);
    add_item(
        &app,
        long_name.clone(),
        &None,
        None,
        Some("Test".to_string()),
    )
    .await?;

    let lists = TodoList::get_all(&app.pool).await?;
    let items = lists[0].get_all_items(&app.pool).await?;
    assert_eq!(items[0].name, long_name);

    Ok(())
}

#[tokio::test]
async fn test_delete_list_with_items() -> Result<()> {
    let app = setup_test_app().await?;

    add_list(&app, "To Delete".to_string(), &None).await?;
    add_item(
        &app,
        "Item 1".to_string(),
        &None,
        None,
        Some("To Delete".to_string()),
    )
    .await?;
    add_item(
        &app,
        "Item 2".to_string(),
        &None,
        None,
        Some("To Delete".to_string()),
    )
    .await?;

    let lists = TodoList::get_all(&app.pool).await?;
    assert_eq!(lists[0].get_all_items(&app.pool).await?.len(), 2);

    delete_list(&app, Some("To Delete".to_string()), None, &None).await?;

    assert_eq!(TodoList::get_all(&app.pool).await?.len(), 0);

    Ok(())
}

#[tokio::test]
async fn test_multiple_lists_with_items() -> Result<()> {
    let app = setup_test_app().await?;

    add_list(&app, "List 1".to_string(), &None).await?;
    add_list(&app, "List 2".to_string(), &None).await?;

    add_item(
        &app,
        "L1 Item 1".to_string(),
        &None,
        None,
        Some("List 1".to_string()),
    )
    .await?;
    add_item(
        &app,
        "L1 Item 2".to_string(),
        &None,
        None,
        Some("List 1".to_string()),
    )
    .await?;
    add_item(
        &app,
        "L2 Item 1".to_string(),
        &None,
        None,
        Some("List 2".to_string()),
    )
    .await?;

    let lists = TodoList::get_all(&app.pool).await?;
    let list1 = lists.iter().find(|l| l.name == "List 1").unwrap();
    let list2 = lists.iter().find(|l| l.name == "List 2").unwrap();

    assert_eq!(list1.get_all_items(&app.pool).await?.len(), 2);
    assert_eq!(list2.get_all_items(&app.pool).await?.len(), 1);

    Ok(())
}

#[tokio::test]
async fn test_list_isolation() -> Result<()> {
    let app = setup_test_app().await?;

    add_list(&app, "List A".to_string(), &None).await?;
    add_list(&app, "List B".to_string(), &None).await?;

    add_item(
        &app,
        "Item A1".to_string(),
        &None,
        None,
        Some("List A".to_string()),
    )
    .await?;
    add_item(
        &app,
        "Item B1".to_string(),
        &None,
        None,
        Some("List B".to_string()),
    )
    .await?;

    let lists = TodoList::get_all(&app.pool).await?;
    let list_a = lists.iter().find(|l| l.name == "List A").unwrap();
    let list_b = lists.iter().find(|l| l.name == "List B").unwrap();

    // Delete item from List A
    let items_a = list_a.get_all_items(&app.pool).await?;
    delete_item(&app, items_a[0].id, &None).await?;

    // Verify List A empty, List B untouched
    assert_eq!(list_a.get_all_items(&app.pool).await?.len(), 0);
    let items_b = list_b.get_all_items(&app.pool).await?;
    assert_eq!(items_b.len(), 1);
    assert_eq!(items_b[0].name, "Item B1");

    Ok(())
}

// ===== Integration / Workflow Tests =====

#[tokio::test]
async fn test_complex_workflow() -> Result<()> {
    let app = setup_test_app().await?;

    // Create a shopping list with items
    add_list(&app, "Shopping".to_string(), &None).await?;
    for name in ["Milk", "Bread", "Eggs"] {
        add_item(
            &app,
            name.to_string(),
            &None,
            None,
            Some("Shopping".to_string()),
        )
        .await?;
    }

    // Mark Milk as done
    let lists = TodoList::get_all(&app.pool).await?;
    let items = lists[0].get_all_items(&app.pool).await?;
    let milk_id = items.iter().find(|i| i.name == "Milk").unwrap().id;
    toggle_done_item(&app, milk_id, &None).await?;

    // Delete Bread
    let bread_id = items.iter().find(|i| i.name == "Bread").unwrap().id;
    delete_item(&app, bread_id, &None).await?;

    // Verify final state
    let final_items = lists[0].get_all_items(&app.pool).await?;
    assert_eq!(final_items.len(), 2);

    let milk = final_items.iter().find(|i| i.name == "Milk").unwrap();
    let eggs = final_items.iter().find(|i| i.name == "Eggs").unwrap();
    assert!(milk.is_done);
    assert!(!eggs.is_done);

    Ok(())
}

#[tokio::test]
async fn test_multiple_operations_on_same_item() -> Result<()> {
    let app = setup_test_app().await?;

    add_list(&app, "Test".to_string(), &None).await?;
    add_item(
        &app,
        "Test Item".to_string(),
        &None,
        None,
        Some("Test".to_string()),
    )
    .await?;

    let lists = TodoList::get_all(&app.pool).await?;
    let item_id = lists[0].get_all_items(&app.pool).await?[0].id;

    // Toggle 3 times: false -> true -> false -> true
    toggle_done_item(&app, item_id, &None).await?;
    toggle_done_item(&app, item_id, &None).await?;
    toggle_done_item(&app, item_id, &None).await?;

    let item = TodoItem::get_by_id(&app.pool, item_id).await?.unwrap();
    assert!(item.is_done);

    Ok(())
}

#[tokio::test]
async fn test_empty_database_operations() -> Result<()> {
    let app = setup_test_app().await?;

    assert_eq!(TodoList::get_all(&app.pool).await?.len(), 0);

    // Add then immediately delete
    add_list(&app, "Temporary".to_string(), &None).await?;
    delete_list(&app, Some("Temporary".to_string()), None, &None).await?;

    assert_eq!(TodoList::get_all(&app.pool).await?.len(), 0);
    Ok(())
}

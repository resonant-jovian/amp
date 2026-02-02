# UI Components Documentation

This document describes the interactive UI components added to the Android app for user confirmations, information display, and settings management.

## Overview

Three new UI components have been added to enhance user interaction:

1. **Confirmation Dialog** - Prevents accidental address deletions
2. **Info Dialog** - Displays detailed address information
3. **Settings Dropdown** - Provides access to app configuration

All components follow the app's neumorphic design system with smooth animations and consistent styling.

## Component Architecture

### File Organization

```
android/src/ui/
├── confirm_dialog.rs    # Confirmation popup UI component
├── info_dialog.rs       # Address info popup UI component
├── settings_dropdown.rs # Settings panel UI component
├── addresses.rs         # Updated with dialog integrations
├── top_bar.rs           # Updated with settings dropdown
└── mod.rs               # Updated module exports

android/assets/
└── style.css            # Updated with modal and dropdown styles
```

### Design Principles

- **UI components only** - No business logic, pure presentation
- **State management** - Parent components manage visibility state
- **Event handlers** - Components emit events, parents handle actions
- **Neumorphic design** - Consistent shadows, colors, and animations
- **Accessibility** - Proper focus management and tap targets

## Components

### 1. Confirmation Dialog

**File:** `android/src/ui/confirm_dialog.rs`

**Purpose:** Prevents accidental destructive actions (e.g., removing addresses)

**Props:**
- `is_open: bool` - Controls visibility
- `title: String` - Dialog header text
- `message: String` - Confirmation message
- `on_confirm: EventHandler<()>` - Called when user confirms
- `on_cancel: EventHandler<()>` - Called when user cancels

**Usage Example:**
```rust
let mut show_confirm = use_signal(|| false);
let mut pending_remove_id = use_signal(|| None::<usize>);

// Show confirmation
let handle_remove_click = move |addr_id: usize| {
    pending_remove_id.set(Some(addr_id));
    show_confirm.set(true);
};

// Handle confirmation
let handle_confirm = move |_| {
    if let Some(id) = pending_remove_id() {
        on_remove_address.call(id);
    }
    show_confirm.set(false);
};

rsx! {
    ConfirmDialog {
        is_open: show_confirm(),
        title: "Bekräfta borttagning".to_string(),
        message: "Är du säker?".to_string(),
        on_confirm: handle_confirm,
        on_cancel: move |_| show_confirm.set(false),
    }
}
```

**Styling:**
- Modal overlay with backdrop blur
- Gradient header with pink accent
- Two buttons: Cancel (neumorphic) and Confirm (gradient)
- Smooth slide-up animation

### 2. Info Dialog

**File:** `android/src/ui/info_dialog.rs`

**Purpose:** Displays detailed information about a stored address

**Props:**
- `is_open: bool` - Controls visibility
- `address: Option<StoredAddress>` - Address to display
- `on_close: EventHandler<()>` - Called when user closes dialog

**Usage Example:**
```rust
let mut show_info = use_signal(|| false);
let mut selected_address = use_signal(|| None::<StoredAddress>);

// Show info
let handle_info_click = move |addr: StoredAddress| {
    selected_address.set(Some(addr));
    show_info.set(true);
};

rsx! {
    InfoDialog {
        is_open: show_info(),
        address: selected_address(),
        on_close: move |_| show_info.set(false),
    }
}
```

**Displayed Information:**
- Street name
- Street number
- Postal code
- Active status (color-coded)
- Validation status
- Taxa (if available)
- Zone (if available)

**Styling:**
- Teal gradient header
- Close button (×) in header
- Grid layout for label-value pairs
- Status indicators with color coding
- Click overlay to close

### 3. Settings Dropdown

**File:** `android/src/ui/settings_dropdown.rs`

**Purpose:** Provides access to app configuration and information

**Props:**
- `is_open: bool` - Controls visibility
- `on_close: EventHandler<()>` - Called when user closes panel

**Usage Example:**
```rust
let mut show_settings = use_signal(|| false);

rsx! {
    button {
        onclick: move |_| show_settings.set(!show_settings()),
        "Settings"
    }
    
    SettingsDropdown {
        is_open: show_settings(),
        on_close: move |_| show_settings.set(false),
    }
}
```

**Menu Items:**
1. Allmänna inställningar (General settings)
2. Aviseringar (Notifications)
3. Om appen (About)
4. Version display (1.0.0)

**Styling:**
- Slides in from top-right
- Pink gradient header
- Neumorphic menu items with icons
- Semi-transparent overlay
- Mobile-responsive width

## CSS Styles

### Modal System

**Overlay:**
```css
.modal-overlay {
    position: fixed;
    background: rgba(0, 0, 0, 0.5);
    backdrop-filter: blur(4px);
    z-index: 1000;
    animation: fadeIn 0.2s ease;
}
```

**Container:**
```css
.modal-container {
    background: var(--color-bg);
    border-radius: 20px;
    box-shadow: var(--shadow-xl);
    animation: slideUp 0.3s cubic-bezier(0.34, 1.56, 0.64, 1);
}
```

### Settings Dropdown

```css
.settings-dropdown {
    position: fixed;
    top: 20px;
    right: 20px;
    width: 300px;
    animation: slideInRight 0.3s cubic-bezier(0.34, 1.56, 0.64, 1);
}
```

### Animations

- **fadeIn**: Overlay fade-in (0.2s)
- **slideUp**: Modal slide from bottom (0.3s)
- **slideInRight**: Settings panel slide from right (0.3s)
- **Easing**: Bouncy cubic-bezier for playful feel

## Integration Points

### Addresses Component

**Updated:** `android/src/ui/addresses.rs`

**Changes:**
- Added confirmation dialog state management
- Added info dialog state management
- Updated remove button to show confirmation
- Updated info button to show address details
- Integrated both dialogs into component tree

### Top Bar Component

**Updated:** `android/src/ui/top_bar.rs`

**Changes:**
- Added settings dropdown state management
- Updated settings button to toggle dropdown
- Integrated settings dropdown into component tree
- Added close handler for dropdown

## State Management Pattern

### Modal State

All modals follow this pattern:

```rust
// 1. Define state signals
let mut show_modal = use_signal(|| false);
let mut modal_data = use_signal(|| None::<DataType>);

// 2. Open handler
let handle_open = move |data: DataType| {
    modal_data.set(Some(data));
    show_modal.set(true);
};

// 3. Close handler
let handle_close = move |_| {
    show_modal.set(false);
    modal_data.set(None);
};

// 4. Action handler (if needed)
let handle_action = move |_| {
    // Perform action with modal_data
    handle_close.call(());
};

// 5. Render modal
rsx! {
    ModalComponent {
        is_open: show_modal(),
        data: modal_data(),
        on_close: handle_close,
        on_action: handle_action,
    }
}
```

## Best Practices

### Component Design

1. **Single Responsibility** - Each component has one clear purpose
2. **Pure UI** - No business logic or data fetching
3. **Props-driven** - All state comes from parent via props
4. **Event Emission** - Components emit events, parents handle them
5. **Conditional Rendering** - Return `None` when `is_open` is false

### State Management

1. **Minimal State** - Only store what's necessary
2. **Clear Lifecycle** - Open → Display → Action → Close
3. **Clean Cleanup** - Reset state on close
4. **Parent Ownership** - Parent components own state

### Styling

1. **Consistent Variables** - Use CSS custom properties
2. **Smooth Animations** - Cubic-bezier easing for polish
3. **Responsive Design** - Mobile-first approach
4. **Accessibility** - Proper tap targets and focus states

### Performance

1. **Lazy Rendering** - Only render when visible
2. **Event Stopping** - Prevent event propagation where needed
3. **Minimal Re-renders** - Signals for fine-grained reactivity

## Future Enhancements

### Potential Improvements

1. **Focus Trapping** - Keep focus within modal when open
2. **Escape Key** - Close modals with Escape key
3. **ARIA Attributes** - Improve screen reader support
4. **Animation Preferences** - Respect reduced-motion settings
5. **Stacking Context** - Handle multiple modals
6. **Touch Gestures** - Swipe to close on mobile

### Settings Functionality

The settings menu items currently log to console. Future implementation should:

1. Create settings pages/modals for each item
2. Implement notification preferences
3. Add about page with app info and licenses
4. Store user preferences in local storage

## Testing

### Manual Testing Checklist

**Confirmation Dialog:**
- [ ] Opens when remove button clicked
- [ ] Shows correct address in message (future)
- [ ] Cancel button closes without action
- [ ] Confirm button removes address
- [ ] Clicking overlay closes dialog
- [ ] Animation smooth and centered

**Info Dialog:**
- [ ] Opens when info icon clicked
- [ ] Displays all address fields correctly
- [ ] Status colors match state (active/inactive)
- [ ] Close button works
- [ ] Clicking overlay closes dialog
- [ ] Shows matched data (taxa, zone) if available

**Settings Dropdown:**
- [ ] Opens when settings button clicked
- [ ] Slides in from top-right smoothly
- [ ] All menu items clickable
- [ ] Close button works
- [ ] Clicking overlay closes panel
- [ ] Responsive on mobile devices

### Edge Cases

- Multiple rapid clicks on trigger buttons
- Opening modal while another is open
- Browser window resize during modal display
- Touch vs. mouse interaction differences

## Troubleshooting

### Common Issues

**Modal doesn't appear:**
- Check `is_open` signal value
- Verify z-index is sufficient (1000+)
- Check for CSS conflicts

**Events not firing:**
- Verify event handler connections
- Check for event propagation issues
- Ensure parent has proper handlers

**Styling issues:**
- Check CSS custom properties defined
- Verify class names match CSS
- Check for specificity conflicts

**Animation stuttering:**
- Reduce animation complexity
- Check for layout thrashing
- Use transform instead of top/left

## References

- **Dioxus Documentation**: [https://dioxuslabs.com/](https://dioxuslabs.com/)
- **Neumorphism Design**: [https://neumorphism.io/](https://neumorphism.io/)
- **ARIA Best Practices**: [https://www.w3.org/WAI/ARIA/apg/](https://www.w3.org/WAI/ARIA/apg/)

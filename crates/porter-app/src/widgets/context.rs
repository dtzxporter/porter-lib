use iced::advanced;
use iced::advanced::Layout;
use iced::advanced::Widget;
use iced::advanced::layout::Limits;
use iced::advanced::layout::Node;
use iced::advanced::mouse;
use iced::advanced::overlay;
use iced::advanced::renderer;
use iced::advanced::widget::Tree;
use iced::advanced::widget::tree::State;
use iced::advanced::widget::tree::Tag;

use iced::Element;
use iced::Event;
use iced::Length;
use iced::Rectangle;
use iced::Size;
use iced::Vector;

/// A menu item in a context menu.
pub struct MenuItem<Message: Clone> {
    name: String,
    action: Option<Message>,
    children: Vec<MenuItem<Message>>,
}

impl<Message> MenuItem<Message>
where
    Message: Clone,
{
    /// Creates a [`MenuItem`] with the given name.
    pub fn new(name: String) -> Self {
        Self {
            name,
            action: None,
            children: Vec::new(),
        }
    }

    /// Sets the action when a menu item is pressed.
    pub fn on_press(mut self, action: Message) -> Self {
        self.action = Some(action);
        self
    }

    /// Sets the optional action when a menu item is pressed.
    pub fn on_press_maybe(mut self, action: Option<Message>) -> Self {
        self.action = action;
        self
    }

    /// Creates a [`MenuItem`] that renders as a divider.
    pub const fn divider() -> Self {
        Self {
            name: String::new(),
            action: None,
            children: Vec::new(),
        }
    }

    /// Appends a [`MenuItem`] to this menu item.
    pub fn child(mut self, item: MenuItem<Message>) -> Self {
        self.children.push(item);
        self.action = None;
        self
    }
}

/// A right click context menu.
pub struct Context<'a, Message, Theme, Renderer>
where
    Message: Clone,
{
    content: Element<'a, Message, Theme, Renderer>,
    items: Vec<MenuItem<Message>>,
}

impl<'a, Message, Theme, Renderer> Context<'a, Message, Theme, Renderer>
where
    Message: Clone,
{
    /// Creates a [`Context`] with the given content.
    pub fn new(content: impl Into<Element<'a, Message, Theme, Renderer>>) -> Self {
        Self {
            content: content.into(),
            items: Vec::new(),
        }
    }

    /// Appends a menu item to this context menu.
    pub fn item(mut self, item: MenuItem<Message>) -> Self {
        self.items.push(item);
        self
    }
}

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for Context<'_, Message, Theme, Renderer>
where
    Renderer: advanced::Renderer,
    Message: Clone,
{
    fn tag(&self) -> Tag {
        Tag::of::<bool>()
    }

    fn state(&self) -> State {
        State::new(false)
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(self.content.as_widget())]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(std::slice::from_ref(&self.content));
    }

    fn size(&self) -> Size<Length> {
        self.content.as_widget().size()
    }

    fn layout(&mut self, tree: &mut Tree, renderer: &Renderer, limits: &Limits) -> Node {
        self.content
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits)
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: advanced::Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn advanced::widget::Operation,
    ) {
        self.content
            .as_widget_mut()
            .operate(&mut tree.children[0], layout, renderer, operation);
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &iced::Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        shell: &mut advanced::Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        let right_click_down = tree.state.downcast_mut::<bool>();

        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Right)) => {
                if !shell.is_event_captured() && cursor.is_over(layout.bounds()) {
                    *right_click_down = true;
                    shell.capture_event();
                }
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Right)) => {
                if !shell.is_event_captured()
                    && cursor.is_over(layout.bounds())
                    && *right_click_down
                {
                    if let Some(message) = show_context_menu(&self.items, shell.window()) {
                        shell.publish(message);
                    }
                    shell.capture_event();
                }

                *right_click_down = false;
            }
            Event::Mouse(mouse::Event::ButtonReleased(_))
            | Event::Mouse(mouse::Event::ButtonPressed(_)) => {
                *right_click_down = false;
            }
            _ => {
                // Not used event.
            }
        }

        self.content.as_widget_mut().update(
            &mut tree.children[0],
            event,
            layout,
            cursor,
            renderer,
            shell,
            viewport,
        );
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.content
            .as_widget()
            .mouse_interaction(&tree.children[0], layout, cursor, viewport, renderer)
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        renderer_style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        self.content.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            renderer_style,
            layout,
            cursor,
            viewport,
        );
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        self.content.as_widget_mut().overlay(
            &mut tree.children[0],
            layout,
            renderer,
            viewport,
            translation,
        )
    }
}

impl<'a, Message, Theme, Renderer> From<Context<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a + Clone,
    Theme: 'a,
    Renderer: advanced::Renderer + 'a,
{
    fn from(value: Context<'a, Message, Theme, Renderer>) -> Self {
        Element::new(value)
    }
}

#[cfg(target_os = "windows")]
mod windows {
    use std::ffi::OsStr;
    use std::os::raw::c_void;
    use std::os::windows::ffi::OsStrExt;

    use iced::Window;

    use raw_window_handle::RawWindowHandle;

    use windows_sys::Win32::Foundation::POINT;

    use windows_sys::Win32::UI::WindowsAndMessaging::AppendMenuW;
    use windows_sys::Win32::UI::WindowsAndMessaging::CreatePopupMenu;
    use windows_sys::Win32::UI::WindowsAndMessaging::DestroyMenu;
    use windows_sys::Win32::UI::WindowsAndMessaging::GetCursorPos;
    use windows_sys::Win32::UI::WindowsAndMessaging::MF_GRAYED;
    use windows_sys::Win32::UI::WindowsAndMessaging::MF_POPUP;
    use windows_sys::Win32::UI::WindowsAndMessaging::MF_SEPARATOR;
    use windows_sys::Win32::UI::WindowsAndMessaging::MF_STRING;
    use windows_sys::Win32::UI::WindowsAndMessaging::TPM_RETURNCMD;
    use windows_sys::Win32::UI::WindowsAndMessaging::TPM_RIGHTBUTTON;
    use windows_sys::Win32::UI::WindowsAndMessaging::TrackPopupMenu;

    use super::MenuItem;

    /// Builds a context menu on windows.
    fn menu_builder<Message>(
        menu: *mut std::ffi::c_void,
        items: &[MenuItem<Message>],
        actions: &mut Vec<Message>,
    ) where
        Message: Clone,
    {
        for item in items {
            match (item.name.is_empty(), &item.action, item.children.is_empty()) {
                (true, None, _) => {
                    unsafe { AppendMenuW(menu, MF_SEPARATOR, 0, std::ptr::null()) };
                }
                (false, None, false) => {
                    let submenu = unsafe { CreatePopupMenu() };

                    menu_builder(submenu, &item.children, actions);

                    let name: Vec<u16> = OsStr::new(&item.name)
                        .encode_wide()
                        .chain(Some(0x0))
                        .collect();

                    unsafe { AppendMenuW(menu, MF_POPUP, submenu as _, name.as_ptr()) };
                }
                (false, None, true) => {
                    let name: Vec<u16> = OsStr::new(&item.name)
                        .encode_wide()
                        .chain(Some(0x0))
                        .collect();

                    unsafe { AppendMenuW(menu, MF_STRING | MF_GRAYED, 0, name.as_ptr()) };
                }
                (_, Some(action), _) => {
                    let id = actions.len() + 1;

                    let name: Vec<u16> = OsStr::new(&item.name)
                        .encode_wide()
                        .chain(Some(0x0))
                        .collect();

                    unsafe { AppendMenuW(menu, MF_STRING, id, name.as_ptr()) };

                    actions.push(action.clone());
                }
            }
        }
    }

    /// Shows a context menu on windows.
    pub fn show_context_menu<Message: Clone>(
        items: &[MenuItem<Message>],
        window: &dyn Window,
    ) -> Option<Message> {
        let Ok(RawWindowHandle::Win32(handle)) = window
            .window_handle()
            .map(|x| x.as_raw())
        else {
            return None;
        };

        let menu = unsafe { CreatePopupMenu() };
        let mut menu_actions: Vec<Message> = Vec::with_capacity(items.len());

        menu_builder(menu, items, &mut menu_actions);

        let mut cursor = POINT::default();

        unsafe { GetCursorPos(&mut cursor) };

        let selected = unsafe {
            TrackPopupMenu(
                menu,
                TPM_RETURNCMD | TPM_RIGHTBUTTON,
                cursor.x,
                cursor.y,
                0,
                handle.hwnd.get() as *mut c_void,
                std::ptr::null(),
            )
        };

        unsafe { DestroyMenu(menu) };

        if selected > 0 {
            return menu_actions
                .into_iter()
                .nth(selected as usize - 1);
        }

        None
    }
}

#[cfg(target_os = "windows")]
use windows::*;

#[cfg(not(target_os = "windows"))]
mod unsupported {
    use super::MenuItem;

    use iced::Window;

    /// Shows a context menu on unsupported platforms.
    pub fn show_context_menu<Message: Clone>(
        _items: &[MenuItem<Message>],
        _window: &dyn Window,
    ) -> Option<Message> {
        for _item in _items {
            let _ = _item.name;
        }

        None
    }
}

#[cfg(not(target_os = "windows"))]
use unsupported::*;

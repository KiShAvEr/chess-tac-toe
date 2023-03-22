use dioxus::prelude::*;
use dioxus_router::use_router;

pub fn MainScreen(cx: Scope) -> Element {
  let router = use_router(cx);

  let opened = use_state(cx, || false);
  let _set_opened = opened.setter();

  let lobby_code = use_state(cx, || "".to_owned());
  let set_lobby_code = lobby_code.setter();

  cx.render(rsx! {
    div { class: "main-menu",
        button { onclick: move |_| { router.navigate_to("/game") }, "Against random opponent" }
        button { onclick: move |_| { opened.set(true) }, "Join lobby" }
        button { onclick: move |_| { router.navigate_to("/game/new") }, "Make lobby" }
        match opened.get() {
            true => rsx!{dialog { 
                class: "join-lobby-dialog",
                open: true, 
                div {

                    class: "join-lobby-container",

                    input {
                        oninput: move |ev| { set_lobby_code(ev.value.clone()) },
                        placeholder: "Lobby code"
                    }
                    button {
                        class: "join-lobby-button",
                        onclick: move |_ev| { if !lobby_code.is_empty() { router.navigate_to(format!("/game/{lobby_code}").as_str()) }; opened.set(false) },
                        "Join lobby"
                    }
                }
                button {
                    class: "close-dialog-button",
                    onclick: move |_ev| { opened.set(false) },
                    "Close"
                } 
            }},
            false => rsx!("")
          }
    }
  })
}

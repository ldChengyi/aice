use leptos::prelude::*;

pub(crate) fn render() -> String {
    let page = view! { <PersonaList /> };
    format!("<!doctype html>{}", page.to_html())
}

#[component]
fn PersonaList() -> impl IntoView {
    view! {
        <html lang="zh-CN">
            <head>
                <meta charset="utf-8" />
                <meta name="viewport" content="width=device-width, initial-scale=1" />
                <title>"机器人人设 · Aice"</title>
                <link rel="stylesheet" href="/ui/assets/personas.css" />
            </head>
            <body>
                <header class="site-header">
                    <div class="container masthead">
                        <span class="brand-mark" aria-hidden="true">"A"</span>
                        <span class="brand">"Aice"</span>
                    </div>
                </header>
                <main class="container">
                    <div class="heading">
                        <div>
                            <h1>"机器人人设"</h1>
                            <p>"查看骰娘的角色设定和回复风格。"</p>
                        </div>
                        <span class="preview-label">"示例数据"</span>
                    </div>
                    <section class="list" aria-label="人设列表">
                        <PersonaCard
                            name="呆猫"
                            initial="猫"
                            tone="violet"
                            role="CoC 跑团助手"
                            description="语气轻松，回答简短，偶尔说一声「喵」。"
                            instructions="你叫呆猫，是协助玩家进行 CoC 跑团的骰娘。用中文简洁地回复。对规则不确定时，直接告诉玩家。"
                        />
                        <PersonaCard
                            name="雾灯"
                            initial="雾"
                            tone="teal"
                            role="调查员建卡"
                            description="耐心地逐项确认设定，陪玩家完成调查员角色卡。"
                            instructions="你叫雾灯。帮助玩家梳理年代、职业和技能，优先询问还缺失的角色信息。"
                        />
                        <PersonaCard
                            name="鸦羽"
                            initial="鸦"
                            tone="amber"
                            role="跑团记录"
                            description="用克制的语气整理线索、人物和未解决的问题。"
                            instructions="你叫鸦羽。按人物、地点和线索归纳跑团记录，并标出尚待确认的信息。"
                        />
                    </section>
                </main>
            </body>
        </html>
    }
}

#[component]
fn PersonaCard(
    name: &'static str,
    initial: &'static str,
    tone: &'static str,
    role: &'static str,
    description: &'static str,
    instructions: &'static str,
) -> impl IntoView {
    view! {
        <details class="persona">
            <summary>
                <span class=format!("avatar {tone}") aria-hidden="true">{initial}</span>
                <span class="identity">
                    <span class="name">{name}</span>
                    <span class="role">{role}</span>
                </span>
                <span class="description">{description}</span>
                <span class="details-label">"人设详情"<span class="chevron" aria-hidden="true"></span></span>
            </summary>
            <p class="instructions">{instructions}</p>
        </details>
    }
}

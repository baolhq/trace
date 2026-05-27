<script lang="ts">
    import { settings } from "$lib/stores/settings.svelte";
    import { DEFAULT_SETTINGS } from "$lib/types";
    import type { Settings, SettingsScope } from "$lib/types";
    import Tooltip from "$lib/components/Tooltip.svelte";

    let {
        open = $bindable(false),
        onOpenRawToml,
    }: {
        open: boolean;
        onOpenRawToml: (scope: SettingsScope) => void;
    } = $props();

    type Tab = "appearance" | "editor" | "behavior" | "advanced";

    let scope: SettingsScope = $state("global");
    let activeTab: Tab = $state("appearance");

    const current = $derived(
        scope === "global" ? settings.global : settings.vault,
    );

    function set<K extends keyof Settings>(key: K, value: Settings[K]) {
        settings.set(scope, key, value);
    }

    function isDefault<K extends keyof Settings>(key: K): boolean {
        const def = DEFAULT_SETTINGS[key];
        const cur = current[key];
        if (Array.isArray(def) && Array.isArray(cur)) {
            return JSON.stringify(cur) === JSON.stringify(def);
        }
        return cur === def;
    }

    function reset<K extends keyof Settings>(key: K) {
        settings.resetField(scope, key);
    }

    function handleKeydown(e: KeyboardEvent) {
        if (!open) return;
        if (e.key === "Escape") {
            open = false;
            e.stopPropagation();
        }
    }

    let modalEl: HTMLElement | null = $state(null);

    // Close when clicking outside the modal box. Deferred one tick so the
    // same click that opened the modal doesn't immediately close it.
    $effect(() => {
        if (!open) return;
        let active = false;
        const id = setTimeout(() => {
            active = true;
        }, 0);
        function onDocClick(e: MouseEvent) {
            if (active && modalEl && !modalEl.contains(e.target as Node)) {
                open = false;
            }
        }
        document.addEventListener("click", onDocClick);
        return () => {
            clearTimeout(id);
            document.removeEventListener("click", onDocClick);
        };
    });
</script>

<!-- Must be at top level, not inside {#if} -->
<svelte:window onkeydown={handleKeydown} />

{#if open}
    <div class="backdrop">
        <div
            class="modal"
            role="dialog"
            aria-label="Settings"
            bind:this={modalEl}
        >
            <!-- Header -->
            <div class="header">
                <span class="title">Settings</span>
                <div class="header-actions">
                    <div class="scope-toggle">
                        <button
                            class="scope-btn"
                            class:active={scope === "global"}
                            onclick={() => (scope = "global")}>Global</button
                        >
                        <button
                            class="scope-btn"
                            class:active={scope === "vault"}
                            onclick={() => (scope = "vault")}>This Vault</button
                        >
                    </div>

                    <Tooltip description="Open raw TOML file">
                        <button
                            class="icon-btn"
                            aria-label="Open raw TOML file"
                            onclick={() => {
                                open = false;
                                onOpenRawToml(scope);
                            }}
                        >
                            <svg
                                viewBox="0 0 16 16"
                                fill="none"
                                stroke="currentColor"
                                stroke-width="1.4"
                                width="14"
                                height="14"
                            >
                                <path
                                    d="M2 4h12M2 8h8M2 12h10"
                                    stroke-linecap="round"
                                />
                            </svg>
                        </button>
                    </Tooltip>

                    <button
                        class="icon-btn"
                        onclick={() => (open = false)}
                        aria-label="Close settings"
                    >
                        <svg
                            viewBox="0 0 10 10"
                            stroke="currentColor"
                            stroke-width="1.3"
                            stroke-linecap="round"
                            width="10"
                            height="10"
                        >
                            <line x1="0.5" y1="0.5" x2="9.5" y2="9.5" />
                            <line x1="9.5" y1="0.5" x2="0.5" y2="9.5" />
                        </svg>
                    </button>
                </div>
            </div>

            <!-- Body -->
            <div class="body">
                <nav class="tabs">
                    {#each ["appearance", "editor", "behavior", "advanced"] as Tab[] as tab}
                        <button
                            class="tab-btn"
                            class:active={activeTab === tab}
                            onclick={() => (activeTab = tab)}
                            >{tab.charAt(0).toUpperCase() +
                                tab.slice(1)}</button
                        >
                    {/each}
                </nav>

                <div class="content">
                    {#if activeTab === "appearance"}
                        <h3 class="section-title">Fonts</h3>

                        <div class="row">
                            <span class="row-label">UI font</span>
                            <div class="row-control">
                                <input
                                    class="text-input"
                                    value={current.ui_font}
                                    oninput={(e) =>
                                        set("ui_font", e.currentTarget.value)}
                                />
                                {@render ResetBtn("ui_font")}
                            </div>
                        </div>
                        <div class="row">
                            <span class="row-label">UI font size</span>
                            <div class="row-control">
                                <input
                                    class="number-input"
                                    type="number"
                                    value={current.ui_font_size}
                                    min={8}
                                    max={32}
                                    oninput={(e) => {
                                        const n = parseInt(
                                            e.currentTarget.value,
                                        );
                                        if (!isNaN(n)) set("ui_font_size", n);
                                    }}
                                />
                                {@render ResetBtn("ui_font_size")}
                            </div>
                        </div>
                        <div class="row">
                            <span class="row-label">Content font</span>
                            <div class="row-control">
                                <input
                                    class="text-input"
                                    value={current.content_font}
                                    oninput={(e) =>
                                        set(
                                            "content_font",
                                            e.currentTarget.value,
                                        )}
                                />
                                {@render ResetBtn("content_font")}
                            </div>
                        </div>
                        <div class="row">
                            <span class="row-label">Content font size</span>
                            <div class="row-control">
                                <input
                                    class="number-input"
                                    type="number"
                                    value={current.content_font_size}
                                    min={8}
                                    max={32}
                                    oninput={(e) => {
                                        const n = parseInt(
                                            e.currentTarget.value,
                                        );
                                        if (!isNaN(n))
                                            set("content_font_size", n);
                                    }}
                                />
                                {@render ResetBtn("content_font_size")}
                            </div>
                        </div>
                        <div class="row">
                            <span class="row-label">Mono font</span>
                            <div class="row-control">
                                <input
                                    class="text-input"
                                    value={current.mono_font}
                                    oninput={(e) =>
                                        set("mono_font", e.currentTarget.value)}
                                />
                                {@render ResetBtn("mono_font")}
                            </div>
                        </div>
                        <div class="row">
                            <span class="row-label">Mono font size</span>
                            <div class="row-control">
                                <input
                                    class="number-input"
                                    type="number"
                                    value={current.mono_font_size}
                                    min={8}
                                    max={32}
                                    oninput={(e) => {
                                        const n = parseInt(
                                            e.currentTarget.value,
                                        );
                                        if (!isNaN(n)) set("mono_font_size", n);
                                    }}
                                />
                                {@render ResetBtn("mono_font_size")}
                            </div>
                        </div>

                        <h3 class="section-title" style="margin-top: 1.5rem">
                            Display
                        </h3>

                        <div class="row">
                            <span class="row-label">Theme</span>
                            <div class="row-control">
                                <input
                                    class="text-input"
                                    value={current.theme}
                                    oninput={(e) =>
                                        set("theme", e.currentTarget.value)}
                                />
                                {@render ResetBtn("theme")}
                            </div>
                        </div>
                        <div class="row">
                            <span class="row-label">Zoom level</span>
                            <div class="row-control">
                                <input
                                    class="number-input"
                                    type="number"
                                    value={current.zoom_level}
                                    min={0.5}
                                    max={3}
                                    step={0.1}
                                    oninput={(e) => {
                                        const n = parseFloat(
                                            e.currentTarget.value,
                                        );
                                        if (!isNaN(n)) set("zoom_level", n);
                                    }}
                                />
                                {@render ResetBtn("zoom_level")}
                            </div>
                        </div>
                        <div class="row">
                            <span class="row-label">Content line height</span>
                            <div class="row-control">
                                <select
                                    class="select-input"
                                    value={current.content_line_height}
                                    onchange={(e) =>
                                        set(
                                            "content_line_height",
                                            e.currentTarget
                                                .value as Settings["content_line_height"],
                                        )}
                                >
                                    <option value="compact">Compact</option>
                                    <option value="standard">Standard</option>
                                    <option value="comfortable"
                                        >Comfortable</option
                                    >
                                </select>
                                {@render ResetBtn("content_line_height")}
                            </div>
                        </div>
                        <div class="row">
                            <span class="row-label">Inline title</span>
                            <div class="row-control">
                                {@render Toggle(
                                    "inline_title",
                                    current.inline_title,
                                )}
                                {@render ResetBtn("inline_title")}
                            </div>
                        </div>
                    {:else if activeTab === "editor"}
                        <h3 class="section-title">Links</h3>

                        <div class="row">
                            <span class="row-label"
                                >Show [[ ]] around wikilinks</span
                            >
                            <div class="row-control">
                                {@render Toggle(
                                    "wikilink_brackets",
                                    current.wikilink_brackets,
                                )}
                                {@render ResetBtn("wikilink_brackets")}
                            </div>
                        </div>
                        <div class="row">
                            <span class="row-label">Download remote images</span
                            >
                            <div class="row-control">
                                {@render Toggle(
                                    "download_remote_images",
                                    current.download_remote_images,
                                )}
                                {@render ResetBtn("download_remote_images")}
                            </div>
                        </div>

                        <h3 class="section-title" style="margin-top: 1.5rem">
                            Insert
                        </h3>

                        <div class="row">
                            <span class="row-label">Date format</span>
                            <div class="row-control">
                                <input
                                    class="text-input"
                                    value={current.date_format}
                                    oninput={(e) =>
                                        set(
                                            "date_format",
                                            e.currentTarget.value,
                                        )}
                                />
                                {@render ResetBtn("date_format")}
                            </div>
                        </div>
                        <div class="row">
                            <span class="row-label">Epoch precision</span>
                            <div class="row-control">
                                <select
                                    class="select-input"
                                    value={current.epoch_precision}
                                    onchange={(e) =>
                                        set(
                                            "epoch_precision",
                                            e.currentTarget
                                                .value as Settings["epoch_precision"],
                                        )}
                                >
                                    <option value="seconds"
                                        >Seconds (10 digits)</option
                                    >
                                    <option value="milliseconds"
                                        >Milliseconds (13 digits)</option
                                    >
                                </select>
                                {@render ResetBtn("epoch_precision")}
                            </div>
                        </div>

                        <h3 class="section-title" style="margin-top: 1.5rem">
                            Spellcheck
                        </h3>

                        <div class="row">
                            <span class="row-label">Enable spellcheck</span>
                            <div class="row-control">
                                {@render Toggle(
                                    "spellcheck",
                                    current.spellcheck,
                                )}
                                {@render ResetBtn("spellcheck")}
                            </div>
                        </div>
                        <div class="row">
                            <span class="row-label"
                                >Languages (comma-separated)</span
                            >
                            <div class="row-control">
                                <input
                                    class="text-input"
                                    value={current.spellcheck_languages.join(
                                        ", ",
                                    )}
                                    oninput={(e) =>
                                        set(
                                            "spellcheck_languages",
                                            e.currentTarget.value
                                                .split(",")
                                                .map((s) => s.trim())
                                                .filter(Boolean),
                                        )}
                                />
                                {@render ResetBtn("spellcheck_languages")}
                            </div>
                        </div>
                    {:else if activeTab === "behavior"}
                        <h3 class="section-title">System</h3>

                        <div class="row">
                            <span class="row-label">Close to tray</span>
                            <div class="row-control">
                                {@render Toggle(
                                    "close_to_tray",
                                    current.close_to_tray,
                                )}
                                {@render ResetBtn("close_to_tray")}
                            </div>
                        </div>
                        <div class="row">
                            <span class="row-label">Autostart on login</span>
                            <div class="row-control">
                                {@render Toggle("autostart", current.autostart)}
                                {@render ResetBtn("autostart")}
                            </div>
                        </div>
                        <div class="row">
                            <span class="row-label">Native app bar</span>
                            <div class="row-control">
                                {@render Toggle(
                                    "native_app_bar",
                                    current.native_app_bar,
                                )}
                                {@render ResetBtn("native_app_bar")}
                            </div>
                        </div>
                        <div class="row">
                            <span class="row-label"
                                >Allow multiple instances</span
                            >
                            <div class="row-control">
                                {@render Toggle(
                                    "multiple_instances",
                                    current.multiple_instances,
                                )}
                                {@render ResetBtn("multiple_instances")}
                            </div>
                        </div>

                        <h3 class="section-title" style="margin-top: 1.5rem">
                            Files
                        </h3>

                        <div class="row">
                            <span class="row-label"
                                >Move deleted notes to trash</span
                            >
                            <div class="row-control">
                                {@render Toggle(
                                    "soft_delete",
                                    current.soft_delete,
                                )}
                                {@render ResetBtn("soft_delete")}
                            </div>
                        </div>
                        <div class="row">
                            <span class="row-label">Auto backup</span>
                            <div class="row-control">
                                {@render Toggle(
                                    "auto_backup",
                                    current.auto_backup,
                                )}
                                {@render ResetBtn("auto_backup")}
                            </div>
                        </div>
                        <div class="row">
                            <span class="row-label">Backup frequency</span>
                            <div class="row-control">
                                <select
                                    class="select-input"
                                    value={current.backup_frequency}
                                    onchange={(e) =>
                                        set(
                                            "backup_frequency",
                                            e.currentTarget
                                                .value as Settings["backup_frequency"],
                                        )}
                                >
                                    <option value="daily">Daily</option>
                                    <option value="weekly">Weekly</option>
                                    <option value="monthly">Monthly</option>
                                </select>
                                {@render ResetBtn("backup_frequency")}
                            </div>
                        </div>
                    {:else if activeTab === "advanced"}
                        <h3 class="section-title">Zen mode</h3>

                        {#each [["zen_app_bar", "App bar"], ["zen_sidebar", "Sidebar"], ["zen_right_panel", "Right panel"], ["zen_status_bar", "Status bar"]] as [keyof Settings, string][] as [key, label]}
                            <div class="row">
                                <span class="row-label">{label}</span>
                                <div class="row-control">
                                    <select
                                        class="select-input"
                                        value={current[key] as string}
                                        onchange={(e) =>
                                            set(
                                                key,
                                                e.currentTarget
                                                    .value as Settings[typeof key],
                                            )}
                                    >
                                        <option value="show">Show</option>
                                        <option value="peek">Peek</option>
                                        <option value="hide">Hide</option>
                                    </select>
                                    {@render ResetBtn(key)}
                                </div>
                            </div>
                        {/each}

                        <h3 class="section-title" style="margin-top: 1.5rem">
                            Editor
                        </h3>

                        <div class="row">
                            <span class="row-label"
                                >I know how to exit Vim mode</span
                            >
                            <div class="row-control">
                                {@render Toggle(
                                    "vim_escaper",
                                    current.vim_escaper,
                                )}
                                {@render ResetBtn("vim_escaper")}
                            </div>
                        </div>
                    {/if}
                </div>
            </div>
        </div>
    </div>
{/if}

<!-- ── Snippets ───────────────────────────────────────────────────────────── -->

{#snippet ResetBtn(key: keyof Settings)}
    <Tooltip description="Reset to default">
        <button
            class="reset-btn"
            class:visible={!isDefault(key)}
            onclick={() => reset(key)}
            tabindex="-1"
            title="Reset to default"
            aria-label="Reset to default"
        >
            <svg
                viewBox="0 0 16 16"
                fill="none"
                stroke="currentColor"
                stroke-width="1.5"
                stroke-linecap="round"
                stroke-linejoin="round"
                width="12"
                height="12"
            >
                <path d="M2.5 8a5.5 5.5 0 1 0 1-3.2" />
                <polyline points="2.5 2 2.5 5.5 6 5.5" />
            </svg>
        </button>
    </Tooltip>
{/snippet}

{#snippet Toggle(key: keyof Settings, checked: boolean)}
    <button
        class="toggle"
        class:on={checked}
        role="switch"
        aria-checked={checked}
        aria-label={String(key).replace(/_/g, " ")}
        onclick={() => set(key, !checked as Settings[typeof key])}
    >
        <span class="thumb"></span>
    </button>
{/snippet}

<style>
    .backdrop {
        position: fixed;
        inset: 0;
        z-index: 200;
        display: flex;
        align-items: center;
        justify-content: center;
        pointer-events: none;
        backdrop-filter: blur(2px);
    }

    .modal {
        pointer-events: auto;
        width: 720px;
        max-width: calc(100vw - 48px);
        height: 70vh;
        max-height: 640px;
        background: var(--bg-primary);
        border: 1px solid var(--bg-border);
        border-radius: 8px;
        display: flex;
        flex-direction: column;
        box-shadow: 0 16px 48px rgba(0, 0, 0, 0.6);
        overflow: hidden;
    }

    /* ── Header ── */

    .header {
        display: flex;
        align-items: center;
        justify-content: space-between;
        padding: 0 1rem;
        height: 44px;
        flex-shrink: 0;
        border-bottom: 1px solid var(--bg-border);
    }

    .title {
        font-size: 0.78rem;
        font-weight: 600;
        letter-spacing: 0.06em;
        text-transform: uppercase;
        color: var(--cursor);
    }

    .header-actions {
        display: flex;
        align-items: center;
        gap: 0.5rem;
    }

    .scope-toggle {
        display: flex;
        background: var(--bg-hover);
        border: 1px solid var(--bg-border);
        border-radius: 4px;
        overflow: hidden;
    }

    .scope-btn {
        padding: 0.2rem 0.65rem;
        font-size: 0.75rem;
        background: none;
        border: none;
        color: var(--cursor);
        cursor: pointer;
        transition:
            background 0.1s,
            color 0.1s;
    }

    .scope-btn.active {
        background: var(--bg-active);
        color: var(--fg-muted);
    }

    .icon-btn {
        width: 26px;
        height: 26px;
        display: flex;
        align-items: center;
        justify-content: center;
        background: none;
        border: none;
        border-radius: 4px;
        color: var(--cursor);
        cursor: pointer;
        transition:
            background 0.1s,
            color 0.1s;
    }

    .icon-btn:hover {
        background: var(--bg-hover);
        color: var(--fg-muted);
    }

    /* ── Body ── */

    .body {
        display: flex;
        flex: 1;
        min-height: 0;
    }

    /* ── Tabs sidebar ── */

    .tabs {
        width: 140px;
        flex-shrink: 0;
        border-right: 1px solid var(--bg-border);
        padding: 0.5rem 0;
        display: flex;
        flex-direction: column;
        gap: 1px;
    }

    .tab-btn {
        width: 100%;
        text-align: left;
        background: none;
        border: none;
        padding: 0.45rem 1rem;
        font-size: 0.82rem;
        color: var(--fg-muted);
        cursor: pointer;
        transition:
            background 0.1s,
            color 0.1s;
    }

    .tab-btn:hover {
        background: var(--bg-hover);
        color: var(--fg-primary);
    }

    .tab-btn.active {
        background: var(--bg-active);
        color: var(--fg-primary);
    }

    /* ── Content area ── */

    .content {
        flex: 1;
        min-width: 0;
        overflow-y: auto;
        overscroll-behavior: contain;
        padding: 1rem 1.25rem;
        scrollbar-width: thin;
        scrollbar-color: var(--bg-border) transparent;
    }

    .section-title {
        font-size: 0.65rem;
        font-weight: 600;
        letter-spacing: 0.1em;
        text-transform: uppercase;
        color: var(--cursor);
        margin: 0 0 0.6rem;
    }

    /* ── Setting row ── */

    .row {
        display: flex;
        align-items: center;
        justify-content: space-between;
        min-height: 36px;
        padding: 0.15rem 0;
        border-bottom: 1px solid
            color-mix(in srgb, var(--bg-border) 40%, transparent);
    }

    .row:last-child {
        border-bottom: none;
    }

    .row-label {
        font-size: 0.82rem;
        color: var(--fg-muted);
        flex: 1;
        min-width: 0;
        padding-right: 1rem;
    }

    .row-control {
        display: flex;
        align-items: center;
        gap: 0.4rem;
        flex-shrink: 0;
    }

    /* ── Controls ── */

    .text-input {
        width: 200px;
        height: 26px;
        padding: 0 0.5rem;
        background: var(--bg-hover);
        border: 1px solid var(--bg-border);
        border-radius: 4px;
        color: var(--fg-primary);
        font-size: 0.8rem;
        outline: none;
        transition: border-color 0.1s;
    }

    .text-input:focus {
        border-color: var(--fg-interactive);
    }

    .number-input {
        width: 72px;
        height: 26px;
        padding: 0 0.4rem;
        background: var(--bg-hover);
        border: 1px solid var(--bg-border);
        border-radius: 4px;
        color: var(--fg-primary);
        font-size: 0.8rem;
        outline: none;
        text-align: right;
        transition: border-color 0.1s;
    }

    .number-input:focus {
        border-color: var(--fg-interactive);
    }

    .select-input {
        height: 26px;
        padding: 0 0.4rem;
        background: var(--bg-hover);
        border: 1px solid var(--bg-border);
        border-radius: 4px;
        color: var(--fg-primary);
        font-size: 0.8rem;
        outline: none;
        cursor: pointer;
        min-width: 130px;
    }

    .select-input:focus {
        border-color: var(--fg-interactive);
    }

    /* ── Toggle ── */

    .toggle {
        width: 36px;
        height: 20px;
        background: var(--bg-active);
        border: 1px solid var(--bg-border);
        border-radius: 10px;
        padding: 0;
        cursor: pointer;
        position: relative;
        transition:
            background 0.15s,
            border-color 0.15s;
        flex-shrink: 0;
    }

    .toggle.on {
        background: var(--fg-interactive);
        border-color: var(--fg-interactive);
    }

    .thumb {
        position: absolute;
        top: 2px;
        left: 2px;
        width: 14px;
        height: 14px;
        background: var(--cursor);
        border-radius: 50%;
        transition:
            left 0.15s,
            background 0.15s;
    }

    .toggle.on .thumb {
        left: 18px;
        background: #fff;
    }

    /* ── Reset button ── */

    .reset-btn {
        width: 20px;
        height: 20px;
        display: flex;
        align-items: center;
        justify-content: center;
        background: none;
        border: none;
        border-radius: 3px;
        color: var(--cursor);
        cursor: pointer;
        opacity: 0;
        pointer-events: none;
        transition:
            opacity 0.1s,
            color 0.1s,
            background 0.1s;
        flex-shrink: 0;
    }

    .reset-btn.visible {
        opacity: 1;
        pointer-events: auto;
    }

    .reset-btn:hover {
        background: var(--bg-hover);
        color: var(--fg-warning);
    }
</style>

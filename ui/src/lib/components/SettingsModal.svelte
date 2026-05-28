<script lang="ts">
    import { invoke } from "@tauri-apps/api/core";
    import { settings } from "$lib/stores/settings.svelte";
    import { DEFAULT_SETTINGS } from "$lib/types";
    import type { Settings, SettingsScope } from "$lib/types";
    import Tooltip from "$lib/components/Tooltip.svelte";
    import CustomSelect from "$lib/components/CustomSelect.svelte";
    import { clickOutside } from "$lib/actions";
    import { positionDropdown } from "$lib/utils/dropdown";

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
        if (!open || e.key !== "Escape") return;
        open = false;
        e.stopPropagation();
    }

    let modalEl: HTMLElement | null = $state(null);

    $effect(() => {
        if (!open) return;
        let active = false;
        const id = setTimeout(() => {
            active = true;
        }, 0);
        function onDocClick(e: MouseEvent) {
            if (active && modalEl && !e.composedPath().includes(modalEl)) {
                open = false;
            }
        }
        document.addEventListener("click", onDocClick);
        return () => {
            clearTimeout(id);
            document.removeEventListener("click", onDocClick);
        };
    });

    // ── Fonts ─────────────────────────────────────────────────────────────────

    let systemFonts: string[] = $state([]);
    let fontsLoaded = false;

    $effect(() => {
        if (!open || fontsLoaded) return;
        fontsLoaded = true;
        invoke<string[]>("list_system_fonts")
            .then((fonts) => {
                systemFonts = fonts;
            })
            .catch(() => {
                systemFonts = [];
            });
    });

    function getFontName(cssValue: string): string {
        return cssValue
            .split(",")[0]
            .trim()
            .replace(/^['"]|['"]$/g, "");
    }

    function buildFont(name: string, fallback: string): string {
        if (!name) return fallback;
        return name.includes(" ")
            ? `'${name}', ${fallback}`
            : `${name}, ${fallback}`;
    }

    // ── Static option lists ───────────────────────────────────────────────────

    const THEMES = [{ value: "trace-dark", label: "Trace Dark" }];

    const LINE_HEIGHT_OPTIONS = [
        { value: "compact", label: "Compact" },
        { value: "standard", label: "Standard" },
        { value: "comfortable", label: "Comfortable" },
    ];

    const DATE_FORMATS = [
        { value: "MMMM d, yyyy", label: "January 5, 2024" },
        { value: "MMM d, yyyy", label: "Jan 5, 2024" },
        { value: "yyyy-MM-dd", label: "2024-01-05" },
        { value: "MM/dd/yyyy", label: "01/05/2024" },
        { value: "dd/MM/yyyy", label: "05/01/2024" },
        { value: "d MMMM yyyy", label: "5 January 2024" },
        { value: "yyyy/MM/dd", label: "2024/01/05" },
    ];

    const EPOCH_OPTIONS = [
        { value: "seconds", label: "Seconds (10 digits)" },
        { value: "milliseconds", label: "Milliseconds (13 digits)" },
    ];

    const BACKUP_OPTIONS = [
        { value: "daily", label: "Daily" },
        { value: "weekly", label: "Weekly" },
        { value: "monthly", label: "Monthly" },
    ];

    const ZEN_OPTIONS = [
        { value: "show", label: "Show" },
        { value: "peek", label: "Peek" },
        { value: "hide", label: "Hide" },
    ];

    // ── Number stepper ────────────────────────────────────────────────────────

    function stepNum(
        key: keyof Settings,
        val: number,
        dir: 1 | -1,
        step: number,
        min: number,
        max: number,
    ) {
        const decimals = step < 1 ? 2 : 0;
        const next = parseFloat((val + dir * step).toFixed(decimals));
        set(key, Math.min(max, Math.max(min, next)) as Settings[typeof key]);
    }

    function commitNumber(
        key: keyof Settings,
        rawValue: string,
        step: number,
        min: number,
        max: number,
    ) {
        const n = parseFloat(rawValue);
        if (!isNaN(n)) {
            const decimals = step < 1 ? 2 : 0;
            set(
                key,
                Math.min(
                    max,
                    Math.max(min, parseFloat(n.toFixed(decimals))),
                ) as Settings[typeof key],
            );
        }
    }

    function fmtNum(val: number, step: number): string {
        return step < 1 ? String(parseFloat(val.toFixed(2))) : String(val);
    }

    // ── Languages ─────────────────────────────────────────────────────────────

    const LANGUAGES = [
        { code: "en-US", label: "English (US)" },
        { code: "en-GB", label: "English (UK)" },
        { code: "en-AU", label: "English (AU)" },
        { code: "fr", label: "French" },
        { code: "de", label: "German" },
        { code: "es", label: "Spanish" },
        { code: "it", label: "Italian" },
        { code: "pt-BR", label: "Portuguese (BR)" },
        { code: "pt-PT", label: "Portuguese (PT)" },
        { code: "nl", label: "Dutch" },
        { code: "pl", label: "Polish" },
        { code: "sv", label: "Swedish" },
        { code: "da", label: "Danish" },
        { code: "fi", label: "Finnish" },
        { code: "nb", label: "Norwegian" },
        { code: "ru", label: "Russian" },
        { code: "uk", label: "Ukrainian" },
        { code: "ja", label: "Japanese" },
        { code: "zh-CN", label: "Chinese (Simplified)" },
        { code: "ko", label: "Korean" },
        { code: "ar", label: "Arabic" },
    ];

    const availableLangs = $derived(
        LANGUAGES.filter((l) => !current.spellcheck_languages.includes(l.code)),
    );

    let langOpen = $state(false);
    let langEl: HTMLElement | null = $state(null);
    let langTriggerEl: HTMLButtonElement | null = $state(null);
    let langPos = $state({ top: 0, right: 0 });

    function openLang() {
        if (langOpen) {
            langOpen = false;
            return;
        }
        if (langTriggerEl) {
            const p = positionDropdown(
                langTriggerEl,
                Math.min(availableLangs.length * 28 + 10, 216),
            );
            langPos = { top: p.top, right: p.right };
        }
        langOpen = true;
    }

    $effect(() => {
        void activeTab;
        langOpen = false;
    });

    function toggleLang(code: string) {
        const langs = current.spellcheck_languages;
        set(
            "spellcheck_languages",
            langs.includes(code)
                ? langs.filter((l) => l !== code)
                : [...langs, code],
        );
    }
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
                            {@render ResetBtn("ui_font")}
                            <div class="row-control">
                                {@render FontSelect("ui_font", "serif")}
                            </div>
                        </div>
                        <div class="row">
                            <span class="row-label">UI font size</span>
                            {@render ResetBtn("ui_font_size")}
                            <div class="row-control">
                                {@render Stepper(
                                    "ui_font_size",
                                    current.ui_font_size,
                                    8,
                                    32,
                                    1,
                                )}
                            </div>
                        </div>
                        <div class="row">
                            <span class="row-label">Content font</span>
                            {@render ResetBtn("content_font")}
                            <div class="row-control">
                                {@render FontSelect(
                                    "content_font",
                                    "sans-serif",
                                )}
                            </div>
                        </div>
                        <div class="row">
                            <span class="row-label">Content font size</span>
                            {@render ResetBtn("content_font_size")}
                            <div class="row-control">
                                {@render Stepper(
                                    "content_font_size",
                                    current.content_font_size,
                                    8,
                                    32,
                                    1,
                                )}
                            </div>
                        </div>
                        <div class="row">
                            <span class="row-label">Mono font</span>
                            {@render ResetBtn("mono_font")}
                            <div class="row-control">
                                {@render FontSelect("mono_font", "monospace")}
                            </div>
                        </div>
                        <div class="row">
                            <span class="row-label">Mono font size</span>
                            {@render ResetBtn("mono_font_size")}
                            <div class="row-control">
                                {@render Stepper(
                                    "mono_font_size",
                                    current.mono_font_size,
                                    8,
                                    32,
                                    1,
                                )}
                            </div>
                        </div>

                        <h3 class="section-title" style="margin-top: 1.5rem">
                            Display
                        </h3>

                        <div class="row">
                            <span class="row-label">Theme</span>
                            {@render ResetBtn("theme")}
                            <div class="row-control">
                                <CustomSelect
                                    value={current.theme}
                                    options={THEMES}
                                    onchange={(v) => set("theme", v)}
                                />
                            </div>
                        </div>
                        <div class="row">
                            <span class="row-label">Zoom level</span>
                            {@render ResetBtn("zoom_level")}
                            <div class="row-control">
                                {@render Stepper(
                                    "zoom_level",
                                    current.zoom_level,
                                    0.25,
                                    5.0,
                                    0.25,
                                )}
                            </div>
                        </div>
                        <div class="row">
                            <span class="row-label">Content line height</span>
                            {@render ResetBtn("content_line_height")}
                            <div class="row-control">
                                <CustomSelect
                                    value={current.content_line_height}
                                    options={LINE_HEIGHT_OPTIONS}
                                    onchange={(v) =>
                                        set(
                                            "content_line_height",
                                            v as Settings["content_line_height"],
                                        )}
                                />
                            </div>
                        </div>
                        <div class="row">
                            <span class="row-label">Inline title</span>
                            {@render ResetBtn("inline_title")}
                            <div class="row-control">
                                {@render Toggle(
                                    "inline_title",
                                    current.inline_title,
                                )}
                            </div>
                        </div>
                    {:else if activeTab === "editor"}
                        <h3 class="section-title">Links</h3>

                        <div class="row">
                            <span class="row-label"
                                >Show [[ ]] around wikilinks</span
                            >
                            {@render ResetBtn("wikilink_brackets")}
                            <div class="row-control">
                                {@render Toggle(
                                    "wikilink_brackets",
                                    current.wikilink_brackets,
                                )}
                            </div>
                        </div>
                        <div class="row">
                            <span class="row-label">Download remote images</span
                            >
                            {@render ResetBtn("download_remote_images")}
                            <div class="row-control">
                                {@render Toggle(
                                    "download_remote_images",
                                    current.download_remote_images,
                                )}
                            </div>
                        </div>

                        <h3 class="section-title" style="margin-top: 1.5rem">
                            Insert
                        </h3>

                        <div class="row">
                            <span class="row-label">Date format</span>
                            {@render ResetBtn("date_format")}
                            <div class="row-control">
                                <CustomSelect
                                    value={current.date_format}
                                    options={DATE_FORMATS}
                                    onchange={(v) => set("date_format", v)}
                                    minWidth={160}
                                />
                            </div>
                        </div>
                        <div class="row">
                            <span class="row-label">Epoch precision</span>
                            {@render ResetBtn("epoch_precision")}
                            <div class="row-control">
                                <CustomSelect
                                    value={current.epoch_precision}
                                    options={EPOCH_OPTIONS}
                                    onchange={(v) =>
                                        set(
                                            "epoch_precision",
                                            v as Settings["epoch_precision"],
                                        )}
                                    minWidth={190}
                                />
                            </div>
                        </div>

                        <h3 class="section-title" style="margin-top: 1.5rem">
                            Spellcheck
                        </h3>

                        <div class="row">
                            <span class="row-label">Enable spellcheck</span>
                            {@render ResetBtn("spellcheck")}
                            <div class="row-control">
                                {@render Toggle(
                                    "spellcheck",
                                    current.spellcheck,
                                )}
                            </div>
                        </div>
                        <div class="row lang-row">
                            <span class="row-label">Languages</span>
                            {@render ResetBtn("spellcheck_languages")}
                            <div class="row-control lang-control">
                                {#each current.spellcheck_languages as code}
                                    <span class="lang-chip">
                                        {LANGUAGES.find((l) => l.code === code)
                                            ?.label ?? code}
                                        <button
                                            class="lang-chip-remove"
                                            type="button"
                                            aria-label="Remove"
                                            onclick={() => toggleLang(code)}
                                            >×</button
                                        >
                                    </span>
                                {/each}
                                {#if availableLangs.length > 0}
                                    <div class="lang-picker" bind:this={langEl}>
                                        <button
                                            class="lang-add-btn"
                                            type="button"
                                            aria-label="Add language"
                                            bind:this={langTriggerEl}
                                            onclick={openLang}>+</button
                                        >
                                        {#if langOpen}
                                            <div
                                                class="lang-list"
                                                use:clickOutside={{
                                                    onClose: () =>
                                                        (langOpen = false),
                                                    exclude: [langEl],
                                                    closeOnScroll: true,
                                                }}
                                                style:top="{langPos.top}px"
                                                style:right="{langPos.right}px"
                                            >
                                                {#each availableLangs as { code, label }}
                                                    <button
                                                        class="lang-option"
                                                        type="button"
                                                        onclick={() => {
                                                            toggleLang(code);
                                                            langOpen = false;
                                                        }}>{label}</button
                                                    >
                                                {/each}
                                            </div>
                                        {/if}
                                    </div>
                                {/if}
                            </div>
                        </div>
                    {:else if activeTab === "behavior"}
                        <h3 class="section-title">System</h3>

                        <div class="row">
                            <span class="row-label">Close to tray</span>
                            {@render ResetBtn("close_to_tray")}
                            <div class="row-control">
                                {@render Toggle(
                                    "close_to_tray",
                                    current.close_to_tray,
                                )}
                            </div>
                        </div>
                        <div class="row">
                            <span class="row-label">Autostart on login</span>
                            {@render ResetBtn("autostart")}
                            <div class="row-control">
                                {@render Toggle("autostart", current.autostart)}
                            </div>
                        </div>
                        <div class="row">
                            <span class="row-label">Native app bar</span>
                            {@render ResetBtn("native_app_bar")}
                            <div class="row-control">
                                {@render Toggle(
                                    "native_app_bar",
                                    current.native_app_bar,
                                )}
                            </div>
                        </div>
                        <div class="row">
                            <span class="row-label"
                                >Allow multiple instances</span
                            >
                            {@render ResetBtn("multiple_instances")}
                            <div class="row-control">
                                {@render Toggle(
                                    "multiple_instances",
                                    current.multiple_instances,
                                )}
                            </div>
                        </div>

                        <h3 class="section-title" style="margin-top: 1.5rem">
                            Files
                        </h3>

                        <div class="row">
                            <span class="row-label"
                                >Move deleted notes to trash</span
                            >
                            {@render ResetBtn("soft_delete")}
                            <div class="row-control">
                                {@render Toggle(
                                    "soft_delete",
                                    current.soft_delete,
                                )}
                            </div>
                        </div>
                        <div class="row">
                            <span class="row-label">Auto backup</span>
                            {@render ResetBtn("auto_backup")}
                            <div class="row-control">
                                {@render Toggle(
                                    "auto_backup",
                                    current.auto_backup,
                                )}
                            </div>
                        </div>
                        <div class="row">
                            <span class="row-label">Backup frequency</span>
                            {@render ResetBtn("backup_frequency")}
                            <div class="row-control">
                                <CustomSelect
                                    value={current.backup_frequency}
                                    options={BACKUP_OPTIONS}
                                    onchange={(v) =>
                                        set(
                                            "backup_frequency",
                                            v as Settings["backup_frequency"],
                                        )}
                                />
                            </div>
                        </div>
                    {:else if activeTab === "advanced"}
                        <h3 class="section-title">Zen mode</h3>

                        {#each [["zen_app_bar", "App bar"], ["zen_sidebar", "Sidebar"], ["zen_right_panel", "Right panel"], ["zen_status_bar", "Status bar"]] as [keyof Settings, string][] as [key, label]}
                            <div class="row">
                                <span class="row-label">{label}</span>
                                {@render ResetBtn(key)}
                                <div class="row-control">
                                    <CustomSelect
                                        value={current[key] as string}
                                        options={ZEN_OPTIONS}
                                        onchange={(v) =>
                                            set(key, v as Settings[typeof key])}
                                    />
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
                            {@render ResetBtn("vim_escaper")}
                            <div class="row-control">
                                {@render Toggle(
                                    "vim_escaper",
                                    current.vim_escaper,
                                )}
                            </div>
                        </div>
                    {/if}
                </div>
            </div>
        </div>
    </div>
{/if}

<!-- ── Snippets ───────────────────────────────────────────────────────────── -->

{#snippet FontSelect(key: keyof Settings, fallback: string)}
    {@const name = getFontName(current[key] as string)}
    {@const fontOptions = [
        { value: "", label: `Default (${fallback})` },
        ...(name && name !== fallback && !systemFonts.includes(name)
            ? [{ value: name, label: name }]
            : []),
        ...systemFonts.map((f) => ({ value: f, label: f })),
    ]}
    <CustomSelect
        value={name === fallback ? "" : name}
        options={fontOptions}
        onchange={(v) =>
            set(
                key,
                (v ? buildFont(v, fallback) : fallback) as Settings[typeof key],
            )}
        minWidth={200}
    />
{/snippet}

{#snippet Stepper(
    key: keyof Settings,
    val: number,
    min: number,
    max: number,
    step: number,
)}
    <div class="stepper">
        <button
            class="stepper-btn"
            tabindex="-1"
            aria-label="Decrease"
            onclick={() => stepNum(key, val, -1, step, min, max)}>−</button
        >
        <input
            class="stepper-input"
            type="text"
            inputmode="decimal"
            value={fmtNum(val, step)}
            onblur={(e) =>
                commitNumber(key, e.currentTarget.value, step, min, max)}
            onkeydown={(e) => {
                if (e.key === "Enter") {
                    commitNumber(key, e.currentTarget.value, step, min, max);
                    e.currentTarget.blur();
                    e.stopPropagation();
                } else if (e.key === "Escape") {
                    e.currentTarget.value = fmtNum(val, step);
                    e.currentTarget.blur();
                    e.stopPropagation();
                }
            }}
        />
        <button
            class="stepper-btn"
            tabindex="-1"
            aria-label="Increase"
            onclick={() => stepNum(key, val, 1, step, min, max)}>+</button
        >
    </div>
{/snippet}

{#snippet ResetBtn(key: keyof Settings)}
    <Tooltip description="Reset to default">
        <button
            class="reset-btn"
            class:visible={!isDefault(key)}
            onclick={() => reset(key)}
            tabindex="-1"
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
        padding: 1rem 1.5rem 1rem 2rem;
        scrollbar-width: thin;
        scrollbar-color: var(--bg-border) transparent;
        scrollbar-gutter: stable;
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

    /* ── Language row ── */

    .row.lang-row {
        align-items: flex-start;
        padding-top: 0.45rem;
        padding-bottom: 0.45rem;
    }

    .row.lang-row .row-label {
        padding-top: 0.25rem;
    }

    .lang-control {
        flex-wrap: wrap;
        justify-content: flex-end;
        gap: 0.3rem;
        max-width: 320px;
    }

    /* ── Language chips ── */

    .lang-chip {
        display: inline-flex;
        align-items: center;
        gap: 0.15rem;
        height: 28px;
        padding: 0 0.25rem 0 0.55rem;
        background: var(--bg-active);
        border: 1px solid var(--bg-border);
        border-radius: 100px;
        font-size: 0.75rem;
        color: var(--fg-muted);
        white-space: nowrap;
    }

    .lang-chip-remove {
        display: inline-flex;
        align-items: center;
        justify-content: center;
        width: 16px;
        height: 16px;
        background: none;
        border: none;
        border-radius: 50%;
        color: var(--cursor);
        cursor: pointer;
        font-size: 0.85rem;
        line-height: 1;
        padding: 0;
        transition:
            background 0.1s,
            color 0.1s;
    }

    .lang-chip-remove:hover {
        background: color-mix(in srgb, var(--fg-warning) 15%, transparent);
        color: var(--fg-warning);
    }

    /* ── Language add button & picker ── */

    .lang-picker {
        position: relative;
        display: inline-flex;
    }

    .lang-add-btn {
        display: inline-flex;
        align-items: center;
        justify-content: center;
        width: 22px;
        height: 22px;
        background: none;
        border: 1px dashed var(--bg-border);
        border-radius: 100px;
        color: var(--cursor);
        cursor: pointer;
        font-size: 1rem;
        line-height: 1;
        padding: 0;
        transition:
            border-color 0.1s,
            color 0.1s,
            background 0.1s;
    }

    .lang-add-btn:hover {
        border-color: var(--fg-interactive);
        color: var(--fg-interactive);
        background: color-mix(in srgb, var(--fg-interactive) 8%, transparent);
    }

    .lang-list {
        position: fixed;
        z-index: 300;
        background: var(--bg-panel);
        border: 1px solid var(--bg-border);
        border-radius: 4px;
        box-shadow: 0 4px 16px rgba(0, 0, 0, 0.4);
        min-width: 160px;
        max-height: 200px;
        overflow-y: auto;
        padding: 0.25rem 0;
        scrollbar-width: thin;
        scrollbar-color: var(--bg-border) transparent;
    }

    .lang-option {
        display: block;
        width: 100%;
        padding: 0.3rem 0.75rem;
        background: none;
        border: none;
        color: var(--fg-muted);
        font-size: 0.8rem;
        text-align: left;
        cursor: pointer;
        white-space: nowrap;
        transition:
            background 0.1s,
            color 0.1s;
    }

    .lang-option:hover {
        background: var(--bg-hover);
        color: var(--fg-primary);
    }

    /* ── Number stepper ── */

    .stepper {
        display: flex;
        align-items: stretch;
        height: 26px;
        background: var(--bg-hover);
        border: 1px solid var(--bg-border);
        border-radius: 4px;
        overflow: hidden;
    }

    .stepper-btn {
        width: 24px;
        height: 100%;
        flex-shrink: 0;
        background: none;
        border: none;
        color: var(--fg-muted);
        cursor: pointer;
        font-size: 1rem;
        line-height: 1;
        padding: 2px 0 0;
        display: flex;
        align-items: center;
        justify-content: center;
        transition:
            background 0.1s,
            color 0.1s;
    }

    .stepper-btn:hover {
        background: var(--bg-active);
        color: var(--fg-primary);
    }

    .stepper-input {
        width: 44px;
        height: 26px;
        box-sizing: border-box;
        flex-shrink: 0;
        background: none;
        border: none;
        border-left: 1px solid var(--bg-border);
        border-right: 1px solid var(--bg-border);
        color: var(--fg-primary);
        font-size: 0.8rem;
        text-align: center;
        outline: none;
        padding: 0 0.35rem;
        cursor: text;
        transition:
            background 0.1s,
            border-color 0.1s;
    }

    .stepper-input:hover {
        background: var(--bg-active);
    }

    .stepper-input:focus {
        background: var(--bg-hover);
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
        width: 28px;
        height: 28px;
        display: none;
        align-items: center;
        justify-content: center;
        background: none;
        border: none;
        border-radius: 3px;
        color: var(--cursor);
        cursor: pointer;
        transition:
            color 0.1s,
            background 0.1s;
        flex-shrink: 0;
        margin-right: 4px;
    }

    .reset-btn.visible {
        display: flex;
    }

    .reset-btn:hover {
        background: var(--bg-hover);
        color: var(--fg-warning);
    }
</style>

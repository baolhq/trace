<script lang="ts" generics="T">
    import type {Snippet} from 'svelte';

    const BUFFER = 5;

    let {
        items,
        totalCount,
        itemHeight,
        onLoadMore,
        children,
    }: {
        items: (T | null)[];
        totalCount: number;
        itemHeight: number;
        onLoadMore?: (visibleEnd: number) => void;
        children: Snippet<[T | null, number]>;
    } = $props();

    let viewport: HTMLDivElement | undefined = $state();
    let viewportHeight = $state(0);
    let scrollTop = $state(0);

    const startIdx = $derived(Math.max(0, Math.floor(scrollTop / itemHeight) - BUFFER));
    const endIdx = $derived(
        Math.min(totalCount, Math.ceil((scrollTop + viewportHeight) / itemHeight) + BUFFER)
    );
    const visibleItems = $derived(
        Array.from({length: Math.max(0, endIdx - startIdx)}, (_, i) => ({
            item: items[startIdx + i] ?? null,
            index: startIdx + i,
        }))
    );

    $effect(() => {
        onLoadMore?.(endIdx);
    });

    $effect(() => {
        if (!viewport) return;
        viewportHeight = viewport.clientHeight;
        const ro = new ResizeObserver(([entry]) => {
            viewportHeight = entry.contentRect.height;
        });
        ro.observe(viewport);
        return () => ro.disconnect();
    });
</script>

<div
        bind:this={viewport}
        class="vl-viewport"
        onscroll={(e) => { scrollTop = (e.currentTarget as HTMLDivElement).scrollTop; }}
>
    <div class="vl-sizer" style="height: {totalCount * itemHeight}px">
        {#each visibleItems as {item, index} (index)}
            <div class="vl-row" style="top: {index * itemHeight}px">
                {@render children(item, index)}
            </div>
        {/each}
    </div>
</div>

<style>
    .vl-viewport {
        overflow-y: auto;
        height: 100%;
    }

    .vl-sizer {
        position: relative;
    }

    .vl-row {
        position: absolute;
        left: 0;
        right: 0;
    }
</style>

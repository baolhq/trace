<div class="container">
    <span class="loader"></span>
</div>

<style>
    .container {
        height: 100vh;
        display: grid;
        place-items: center;
    }

    .loader {
        width: 240px;
        height: 240px;
        border: 8px dotted var(--fg-primary);
        border-style: solid solid dotted dotted;
        border-radius: 50%;
        display: inline-block;
        position: relative;
        box-sizing: border-box;
        animation: rotation 2s linear infinite;
    }
    .loader::after {
        content: "";
        box-sizing: border-box;
        position: absolute;
        left: 0;
        right: 0;
        top: 0;
        bottom: 0;
        margin: auto;
        border: 8px dotted var(--fg-success);
        border-style: solid solid dotted;
        width: 200px;
        height: 200px;
        border-radius: 50%;
        animation: rotationBack 1s linear infinite;
        transform-origin: center center;
    }

    @keyframes rotation {
        0% {
            transform: rotate(0deg);
        }
        100% {
            transform: rotate(360deg);
        }
    }
    @keyframes rotationBack {
        0% {
            transform: rotate(0deg);
        }
        100% {
            transform: rotate(-360deg);
        }
    }
</style>

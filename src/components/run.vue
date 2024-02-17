<template>
    <button id="run" class="pointer text" @click="run">启动游戏</button>
</template>

<script lang="ts">
// This starter template is using Vue 3 <script setup> SFCs
// Check out https://vuejs.org/api/sfc-script-setup.html#script-setup
// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command

import { invoke } from "@tauri-apps/api/tauri";
import { listen } from "@tauri-apps/api/event";
import $ from "jquery";

listen("run-progress", (e) => {
    if ((e.payload as string).endsWith("Done") || e.payload == "Already Running") $("#run").text("游戏正在运行");
    else if ((e.payload as string).endsWith("Closed")) $("#run").removeClass("active").text("启动游戏");
});

export default {
    methods: {
        run() {
            if (!$("#run").hasClass("active")) {
                $("#run").addClass("active");
                invoke("unlock_fps");
            }
        }
    }
}
</script>

<style scoped>
#run {
    width: 240px;
    height: 60px;
    background-color: rgba(166,127,120, 0.7);
    backdrop-filter: blur(18px) saturate(150%);
    box-shadow: 3px 3px 10px rgba(0,0,0, 0.7);
    position: absolute;
    font-size: 20px;
    right: 100px;
    bottom: 50px;
}

#run:hover {
    background-color: rgba(166,127,120);
}

#run.active {
    background-color: rgba(55,57,68, 0.7);
    cursor: not-allowed;
}
</style>
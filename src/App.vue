<template>
    <div class="container justify-center align-center flex">
        <div class="left-container">
            <Banner />
            <Post />
        </div>
        <Run />
        <Setting />
    </div>
</template>

<script lang="ts">
// This starter template is using Vue 3 <script setup> SFCs
// Check out https://vuejs.org/api/sfc-script-setup.html#script-setup
// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command

// import { invoke } from "@tauri-apps/api/tauri";
import { invoke } from "@tauri-apps/api/tauri";
import $ from "jquery";
import * as data from "./scripts/data.ts";

import Run from "./components/run.vue";
import Banner from "./components/banner.vue";
import Post from "./components/post.vue";

import Setting from "./components/pages/setting.vue";

export default {
    components: {
        Run, Banner, Post, Setting
    },
    async mounted() {
        invoke('is_admin').then((value: unknown) => data.setAdmin(value as boolean));
        data.bus.on("content-fetched", function(e: any) {
            $(".container").css("backgroundImage",
            `url(${(e as data.GeneralResponse<any>).data.adv?.background})`);
        });
    }
}
</script>

<style scoped>
.container {
    width: 100%;
    height: 100%;
    top: 0;
    left: 0;
    position: absolute;
    overflow: hidden;
    background-repeat: no-repeat;
    background-image: url("./assets/default.png");
}

.left-container {
    position: absolute;
    top: 30%;
    left: 60px;
}
</style>

<style>
.banner, .banner-item, .post {
    width: 430px;
}

.banner, .post, .page, .page-button {
    border-radius: 8px;
    background-color: rgba(55,57,68, 0.7);
    backdrop-filter: blur(18px) saturate(150%);
    box-shadow: 3px 3px 10px rgba(0,0,0, 0.7);
    transition: all 0.2s;
}

.page-button i { font-size: 1.75rem; }

.page-button {
    padding: 8px;
    box-sizing: border-box;
    height: fit-content;
}

.page-button:hover { background-color: rgba(55,57,68, 0.9); }

</style>
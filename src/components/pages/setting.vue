<template>
    <button class="page-button" id="setting" @click="setting = !setting">
        <i class="iconfont icon-shezhi text"></i>
    </button>
    <div class="page setting" :class="{ active: setting }">
        
    </div>
</template>

<script lang="ts">
// This starter template is using Vue 3 <script setup> SFCs
// Check out https://vuejs.org/api/sfc-script-setup.html#script-setup
// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command

// import { defineComponent, ref, onMounted } from 'vue';
import { defineComponent, ref } from 'vue';
import { shell } from "@tauri-apps/api";
// import * as data from "../../scripts/data.ts";

export default defineComponent({
    methods: {
        handleHref(url: string) { shell.open(url); }
    },
    setup() {
        const setting = ref(false);
        return {
            setting
        };
    }
})
</script>

<style scoped>

#setting {
    position: absolute;
    left: 60px;
    bottom: 50px;
}

.setting {
    width: 900px;
    height: 500px;
    opacity: 0;
    margin-top: 5vh;
    position: relative;
    visibility: hidden;
    transition: all 0.5s cubic-bezier(0,1,.6,1);
}

.setting.active {
    margin-top: 0vh;
    visibility: visible;
    opacity: 1;
}

</style>
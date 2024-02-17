<template>
    <div class="post text">
        <div class="post-headers flex-row align-center none-select">
            <button class="post-tab event text pointer" :class="{ active: tab == 0 }" @click="tab = 0">活动</button>
            <button class="post-tab announce text pointer" :class="{ active: tab == 1 }" @click="tab = 1">公告</button>
            <button class="post-tab info text pointer" :class="{ active: tab == 2 }" @click="tab = 2">信息</button>
        </div>
        <div class="cross-split"></div>
        <div class="post-body justify-center">
            <div class="post-body event" v-if="tab == 0">
                <div v-for="post in events" class="post-body-item flex-row pointer none-select"
                :key="post.post_id" @click.prevent="() => handleHref(post.url)">
                    <span class="block ellipsis">{{ post.title }}</span>
                    <span class="block">{{ post.show_time }}</span>
                </div>
            </div>
            <div class="post-body announce" v-if="tab == 1">
                <div v-for="post in announces" class="post-body-item flex-row pointer none-select"
                :key="post.post_id" @click.prevent="() => handleHref(post.url)">
                    <span class="block ellipsis">{{ post.title }}</span>
                    <span class="block">{{ post.show_time }}</span>
                </div>
            </div>
            <div class="post-body info" v-if="tab == 2">
                <div v-for="post in infos" class="post-body-item flex-row pointer none-select"
                :key="post.post_id" @click.prevent="() => handleHref(post.url)">
                    <span class="block ellipsis">{{ post.title }}</span>
                    <span class="block">{{ post.show_time }}</span>
                </div>
            </div>
        </div>
    </div>
</template>

<script lang="ts">
// This starter template is using Vue 3 <script setup> SFCs
// Check out https://vuejs.org/api/sfc-script-setup.html#script-setup
// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command

import { defineComponent, ref, onMounted } from 'vue';
import { shell } from "@tauri-apps/api";
import * as data from "../scripts/data.ts";

export default defineComponent({
    methods: {
        handleHref(url: string) { shell.open(url); }
    },
    setup() {
        const tab = ref(0);
        const events = ref<data.PostResponse[]>([]);
        const announces = ref<data.PostResponse[]>([]);
        const infos = ref<data.PostResponse[]>([]);
        onMounted(() => {
            data.bus.on('content-fetched', (e: any) => {
                const resp = e as data.GeneralResponse<any>;
                events.value = resp.data?.post.filter((b: data.PostResponse) =>
                b.type === data.PostType.POST_TYPE_ACTIVITY);
                announces.value = resp.data?.post.filter((b: data.PostResponse) =>
                b.type === data.PostType.POST_TYPE_ANNOUNCE);
                infos.value = resp.data?.post.filter((b: data.PostResponse) =>
                b.type === data.PostType.POST_TYPE_INFO);
            });
        });
        return {
            tab, events, announces, infos
        };
    }
})
</script>

<style scoped>
.post {
    padding: 4px 14px;
    box-sizing: border-box;
    max-height: 272px;
    min-height: 136px;
    overflow: auto;
}

.post-headers { height: 35px; }

.post-tab {
    font-weight: 700;
    padding: 0 12px;
    position: relative;
}

.post-tab::after {
    position: absolute;
    bottom: -6px;
    left: 10%;
    width: 80%;
    height: 3px;
    background: #5190bb;
    border-radius: 5px;
    opacity: 0;
    transition: all 0.2s;
}

.post-tab.active::after { opacity: 1; }

.post-body-item {
    line-height: 28px;
    font-size: 14px;
}

.post-body-item span.ellipsis { max-width: 350px; }

.post-body-item span:not(.ellipsis) {
    position: absolute;
    right: 14px;
}

.post-tab:hover,
.post-body-item:hover span {
    color: #5190bb;
    transition: all 0.1s;
}
</style>
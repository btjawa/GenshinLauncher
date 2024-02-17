<template>
    <div class="banner">
        <transition-group name="banner-slide" tag="div" class="banner-items">
            <a v-for="banner in banners" :key="banner.banner_id"
                @click.prevent="() => handleHref(banner.url)" class="banner-item pointer"
                :style="{ backgroundImage: `url(${banner.img})`,
                opacity: banner.opacity }">
            </a>
        </transition-group>
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
        const banners = ref<data.BannerResponse[]>([]);
        onMounted(() => {
            data.bus.on('content-fetched', (resp: any) => {
                banners.value = (resp as data.GeneralResponse<any>)
                .data.banner.map((b: any) => ({ ...b, opacity: true }));
                const li = banners.value.pop();
                if (li) banners.value.unshift(li);
                setInterval(() => {
                    const fi = banners.value.shift() as data.BannerResponse;
                    fi.opacity = 0;
                    if (fi) banners.value.push(fi);
                    setTimeout(() => fi.opacity = 1, 500);
                }, 5000);
            });
        });
        return { banners };
    }
})
</script>

<style scoped>
.banner, .banner-item {
    height: 200px;
    transition: all 0.2s;
}

.banner {
    overflow: hidden;
    margin-bottom: 25px;
}

.banner-items {
    display: flex;
    transform: translateX(-100%)
}

.banner-item {
    flex: 0 0 100%;
    background-repeat: no-repeat;
    background-size: cover;
    transition: transform 0.5s cubic-bezier(0,1,1,1);
}

.banner:hover {
    background-color: rgba(55,57,68, 0.8);
}
</style>
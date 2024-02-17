import mitt from 'mitt';

export let bus = mitt();

export let admin: boolean = false;

export function setAdmin(value: boolean) {
    admin = value;
}

import { http } from "@tauri-apps/api";

http.fetch("https://sdk-static.mihoyo.com/hk4e_cn/mdk/launcher/api/content?key=eYd89JmJ&launcher_id=18&language=zh-cn")
.then(resp => bus.emit("content-fetched", resp.data as GeneralResponse<any>));

export enum PostType {
    POST_TYPE_INFO = "POST_TYPE_INFO",
    POST_TYPE_ACTIVITY = "POST_TYPE_ACTIVITY",
    POST_TYPE_ANNOUNCE = "POST_TYPE_ANNOUNCE"
}

export interface GeneralResponse<T> {
    retcode: number,
    message: string,
    data: T;
}

export interface BannerResponse {
    banner_id: string,
    name: string,
    img: string,
    url: string,
    order: string,
    opacity: number
}

export interface PostResponse {
    post_id: string,
    type: PostType,
    tittle: string,
    url: string,
    show_time: string,
    order: string,
    title: string
}
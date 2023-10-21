<script setup lang="ts">
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/tauri";
import SessionState from "./SessionState.vue";

interface Partition {
    start_url: string
    slug: string
    account_id: string
    region: string
    status?: string
    message?: string
}

const partitions = ref<Partition[]>([]);

async function getPartitions() {
    partitions.value = await invoke("get_partitions", {});
}

await getPartitions();
</script>

<template>
    <div v-for="p in partitions">
        <SessionState :partition=p />
    </div>
</template>
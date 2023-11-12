<script setup lang="ts">
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/tauri";
import SessionState from "./SessionState.vue";
import { Partition } from "../store";

const partitions = ref<Partition[]>([]);

async function getPartitions() {
    partitions.value = await invoke("get_partitions", {});
}

await getPartitions();
</script>

<template>
    <div v-for="p in partitions" :key="p.slug">
        <SessionState :partition=p />
    </div>
</template>
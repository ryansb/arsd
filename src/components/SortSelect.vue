<script setup lang="ts">
import { invoke } from "@tauri-apps/api/core";
import { onBeforeMount } from "vue";
import { SortOrder, useSessionStore } from "../store";

const store = useSessionStore();

async function handleToggle() {
  await invoke("settings_save_sort", { sort: store.sort });
}

onBeforeMount(async () => {
  const sort = await invoke("settings_get_sort");
  if (SortOrder[sort as keyof typeof SortOrder] === undefined) {
    console.error(`Invalid sort order from DB: ${sort}`);
    console.warn("Leaving sort as default");
    return;
  }
  store.sort = sort as SortOrder;
});
</script>

<template>
    <VBtnToggle v-model="store.sort" v-on:update:model-value="handleToggle" rounded="pill" class="mx-2" color="primary"
        mandatory>
        <VBtn>
            <VIcon>mdi-sort-alphabetical-ascending</VIcon>
            <VTooltip activator="parent" location="bottom" open-delay="500">Sort accounts by name</VTooltip>
        </VBtn>
        <VBtn>
            <VIcon>mdi-chart-histogram</VIcon>
            <VTooltip activator="parent" location="bottom" open-delay="500">Sort accounts by frequency score</VTooltip>
        </VBtn>
    </VBtnToggle>
</template>
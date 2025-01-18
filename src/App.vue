<script setup lang="ts">
import { ref, watch, nextTick, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";

import SessionList from "./components/SessionList.vue";
import SortSelect from "./components/SortSelect.vue";
import SearchBar from "./components/SearchBar.vue";
import SessionToolbar from "./components/SessionToolbar.vue";
import SettingsToolbar from "./components/SettingsToolbar.vue";

const drawer = ref(false);
const search = ref(false);
const searchBox = ref<HTMLInputElement | null>(null);
const partitionsFound = ref(true);

watch(search, (isShown) => {
  if (isShown) {
    nextTick(() => {
      searchBox.value && searchBox.value.focus();
    });
  }
});
onMounted(async () => {
  if ((((await invoke("get_partitions")) as []) || []).length === 0) {
    partitionsFound.value = false;
  }
});
</script>

<template>
    <VApp>
        <VAppBar scroll-behavior="elevate" elevation="2" color="surface">
            <VAppBarNavIcon @click="drawer = !drawer">
                <VIcon>mdi-menu</VIcon>
            </VAppBarNavIcon>
            <SearchBar />
            <VSpacer />
            <SortSelect />
        </VAppBar>
        <VNavigationDrawer v-model="drawer">
            <VList>
                <Suspense>
                    <SessionToolbar />
                </Suspense>
            </VList>
            <template v-slot:append>
                <SettingsToolbar />
            </template>
        </VNavigationDrawer>
        <VMain>
            <VContainer>
                <VAlert v-if="!partitionsFound" color="error" icon="$error">
                    <VAlertTitle>Missing SSO Partitions</VAlertTitle>
                    No SSO partitions found. Please check your config file and try again. See the <a
                        href="https://github.com/ryansb/arsd#configuration" target="_blank">configuration docs</a> for
                    examples.
                </VAlert>
                <Suspense>
                    <SessionList />
                </Suspense>
            </VContainer>
        </VMain>
    </VApp>
</template>
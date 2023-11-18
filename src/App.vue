<script setup lang="ts">
import { ref, watch } from "vue";
import SessionList from "./components/SessionList.vue";
import SortSelect from "./components/SortSelect.vue";
import SearchBar from "./components/SearchBar.vue";
import { nextTick } from "vue";
import SessionToolbar from "./components/SessionToolbar.vue";
import SettingsToolbar from "./components/SettingsToolbar.vue";

const drawer = ref(false);
const search = ref(false);
const searchBox = ref<HTMLInputElement | null>(null);

watch(search, (isShown) => {
    if (isShown) {
        nextTick(() => {
            searchBox.value && searchBox.value.focus();
        })
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
                <Suspense>
                    <SessionList />
                </Suspense>
            </VContainer>
        </VMain>
    </VApp>
</template>
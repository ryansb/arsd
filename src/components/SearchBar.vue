<script setup lang="ts">
import { ref, watch } from "vue";
import { useSessionStore } from "../store";
import { nextTick } from "vue";

const search = ref(false);
const searchTerms = ref("");
const searchBox = ref<HTMLInputElement | null>(null);
const store = useSessionStore();

const DEBOUNCE_MS = 250;
let timeout: number;

watch(searchTerms, (terms) => {
    timeout && clearTimeout(timeout);

    timeout = setTimeout(() => {
        if (terms && terms.length >= 1) store.$patch({ search_term: terms })
        else store.$patch({ search_term: undefined })
    }, DEBOUNCE_MS);
});

watch(search, (isShown) => {
    if (isShown) {
        nextTick(() => {
            searchBox.value && searchBox.value.focus();
        })
    }
});
</script>

<template>
    <VAppBarNavIcon @click="search = !search; searchTerms = ''">
        <VIcon>mdi-magnify</VIcon>
    </VAppBarNavIcon>
    <VTextField v-if="search" ref="searchBox" v-model="searchTerms" hide-details placeholder="Search" single-line clearable
        color="secondary" persistent-clear />
</template>
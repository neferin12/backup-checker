<script setup lang="ts">
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-dialog'
import Button from 'primevue/button'
import InputText from 'primevue/inputtext'
import InputNumber from 'primevue/inputnumber'
import Dropdown from 'primevue/dropdown'
import Card from 'primevue/card'
import DataTable from 'primevue/datatable'
import Column from 'primevue/column'
import Message from 'primevue/message'
import ProgressSpinner from 'primevue/progressspinner'

const oldFolder = ref('')
const newFolder = ref('')
const maxDepth = ref(10)

const generators = [
  { label: 'CRC32', value: 'crc32' },
  { label: 'SHA256', value: 'sha256' },
  { label: 'Adler32', value: 'adler32' },
  { label: 'MD5', value: 'md5' }
]
const generator = ref('crc32')

const missingFiles = ref<string[]>([])
const loading = ref(false)
const error = ref('')
const hasRun = ref(false)

async function selectFolder(target: 'old' | 'new') {
  try {
    const selected = await open({
      directory: true,
      multiple: false
    })
    
    if (selected && typeof selected === 'string') {
      if (target === 'old') {
        oldFolder.value = selected
      } else {
        newFolder.value = selected
      }
    }
  } catch (e: any) {
    error.value = e.toString()
  }
}

async function checkBackup() {
  if (!oldFolder.value || !newFolder.value) {
    error.value = 'Please select both old and new folders'
    return
  }

  loading.value = true
  error.value = ''
  hasRun.value = false
  missingFiles.value = []

  try {
    missingFiles.value = await invoke('check_backup', {
      oldFolder: oldFolder.value,
      newFolder: newFolder.value,
      maxDepth: maxDepth.value,
      generator: generator.value
    })
    hasRun.value = true
  } catch (e: any) {
    error.value = e.toString()
  } finally {
    loading.value = false
  }
}
</script>

<template>
  <div class="container">
    <Card class="mb-4">
      <template #title>Backup Checker</template>
      <template #content>
        <div class="form-grid">
          <div class="field">
            <label>Old Folder</label>
            <div class="input-group">
              <InputText v-model="oldFolder" readonly placeholder="Select original directory..." class="flex-auto" />
              <Button icon="pi pi-folder-open" @click="selectFolder('old')" />
            </div>
          </div>
          
          <div class="field">
            <label>New Folder</label>
            <div class="input-group">
              <InputText v-model="newFolder" readonly placeholder="Select backup directory..." class="flex-auto" />
              <Button icon="pi pi-folder-open" @click="selectFolder('new')" />
            </div>
          </div>

          <div class="field-row">
            <div class="field flex-1">
              <label>Max Depth</label>
              <InputNumber v-model="maxDepth" :min="1" :max="100" />
            </div>
            <div class="field flex-1">
              <label>Checksum Generator</label>
              <Dropdown v-model="generator" :options="generators" optionLabel="label" optionValue="value" class="w-full" />
            </div>
          </div>
        </div>
      </template>
      <template #footer>
        <Button label="Check Backup" icon="pi pi-check" @click="checkBackup" :loading="loading" class="w-full" />
      </template>
    </Card>

    <Message v-if="error" severity="error" :closable="false" class="mb-4">{{ error }}</Message>

    <Card v-if="loading" class="text-center">
      <template #content>
        <div class="loading-container">
          <ProgressSpinner />
          <p>Calculating checksums and comparing files... This may take a while.</p>
        </div>
      </template>
    </Card>

    <Card v-if="hasRun && !loading">
      <template #title>
        Results
      </template>
      <template #content>
        <Message v-if="missingFiles.length === 0" severity="success" :closable="false">
          All files successfully backed up!
        </Message>
        <div v-else>
          <Message severity="warn" :closable="false" class="mb-3">
            Found {{ missingFiles.length }} missing files in the backup.
          </Message>
          <DataTable :value="missingFiles.map(f => ({ path: f }))" :paginator="true" :rows="10" tableStyle="min-width: 50rem">
            <Column field="path" header="Missing File Path" sortable></Column>
          </DataTable>
        </div>
      </template>
    </Card>
  </div>
</template>

<style scoped>
.container {
  max-width: 800px;
  margin: 0 auto;
  padding: 2rem;
}

.mb-4 {
  margin-bottom: 1.5rem;
}

.mb-3 {
  margin-bottom: 1rem;
}

.form-grid {
  display: flex;
  flex-direction: column;
  gap: 1.5rem;
}

.field {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.field label {
  font-weight: 600;
}

.input-group {
  display: flex;
  gap: 0.5rem;
}

.flex-auto {
  flex: 1 1 auto;
}

.field-row {
  display: flex;
  gap: 1rem;
}

.flex-1 {
  flex: 1;
}

.w-full {
  width: 100%;
}

.text-center {
  text-align: center;
}

.loading-container {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 1rem;
}
</style>
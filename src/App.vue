<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { invoke } from "@tauri-apps/api/core";

type Todo = { id: string; title: string; completed: number; created_at: string; updated_at: string };
type Memo = { id: string; title: string; body: string; created_at: string; updated_at: string };
type View = "todos" | "memos";

const view = ref<View>("todos");
const todos = ref<Todo[]>([]);
const memos = ref<Memo[]>([]);
const todoDraft = ref("");
const editingTodo = ref<string | null>(null);
const editingTodoText = ref("");
const selectedMemo = ref<Memo | null>(null);
const memoTitle = ref("");
const memoBody = ref("");
const loading = ref(true);
const error = ref("");

const remaining = computed(() => todos.value.filter((todo) => !todo.completed).length);
const completed = computed(() => todos.value.filter((todo) => todo.completed).length);

async function load() {
  loading.value = true;
  error.value = "";
  try {
    [todos.value, memos.value] = await Promise.all([
      invoke<Todo[]>("list_todos"),
      invoke<Memo[]>("list_memos"),
    ]);
    if (!selectedMemo.value && memos.value[0]) selectMemo(memos.value[0]);
    if (selectedMemo.value) {
      const current = memos.value.find((memo) => memo.id === selectedMemo.value?.id);
      if (current) selectMemo(current);
    }
  } catch (err) { error.value = String(err); } finally { loading.value = false; }
}

async function addTodo() {
  if (!todoDraft.value.trim()) return;
  await invoke("save_todo", { input: { title: todoDraft.value } });
  todoDraft.value = "";
  await load();
}

async function toggleTodo(todo: Todo) {
  await invoke("set_todo_completed", { id: todo.id, completed: !todo.completed });
  await load();
}

function beginTodoEdit(todo: Todo) { editingTodo.value = todo.id; editingTodoText.value = todo.title; }
async function saveTodoEdit(todo: Todo) {
  if (!editingTodoText.value.trim()) return;
  await invoke("save_todo", { input: { id: todo.id, title: editingTodoText.value } });
  editingTodo.value = null;
  await load();
}
async function removeTodo(id: string) {
  if (!window.confirm("删除这项待办？")) return;
  try { await invoke("delete_todo", { id }); await load(); } catch (err) { error.value = String(err); }
}
async function moveTodo(id: string, direction: -1 | 1) {
  try { await invoke("move_todo", { id, direction }); await load(); } catch (err) { error.value = String(err); }
}

function newMemo() { selectedMemo.value = null; memoTitle.value = ""; memoBody.value = ""; view.value = "memos"; }
function selectMemo(memo: Memo) { selectedMemo.value = memo; memoTitle.value = memo.title; memoBody.value = memo.body; }
async function saveMemo() {
  if (!memoTitle.value.trim() && !memoBody.value.trim()) return;
  const saved = await invoke<Memo>("save_memo", { input: { id: selectedMemo.value?.id, title: memoTitle.value, body: memoBody.value } });
  selectedMemo.value = saved;
  await load();
}
async function removeMemo(id: string) {
  if (!confirm("删除这篇备忘录？")) return;
  await invoke("delete_memo", { id });
  selectedMemo.value = null; memoTitle.value = ""; memoBody.value = ""; await load();
}
function formatDate(value: string) { return value.replace("T", " ").slice(0, 16); }
function focusTodo() { document.querySelector<HTMLInputElement>("#todo-input")?.focus(); }

onMounted(load);
</script>

<template>
  <div class="app-shell">
    <aside class="sidebar">
      <div class="brand"><div class="brand-mark">小小</div><div><strong>小小清单</strong><span>记录每一件要完成的事</span></div></div>
      <nav class="nav-list" aria-label="主导航">
        <button :class="{ active: view === 'todos' }" @click="view = 'todos'"><span class="nav-icon">✓</span><span>待办清单</span><b>{{ remaining }}</b></button>
        <button :class="{ active: view === 'memos' }" @click="view = 'memos'"><span class="nav-icon">≡</span><span>备忘录</span><b>{{ memos.length }}</b></button>
      </nav>
      <div class="sidebar-footer"><div class="flag-line"></div><p>Keep your eyes<br />on the horizon.</p><small>PRIVATE · LOCAL SQLITE</small></div>
    </aside>

    <main class="content">
      <header class="topbar"><div><p class="eyebrow">{{ view === 'todos' ? 'TODAY\'S COURSE' : 'CAPTAIN\'S LOG' }}</p><h1>{{ view === 'todos' ? '待办清单' : '备忘录' }}</h1></div><button class="primary-button" @click="view === 'todos' ? focusTodo() : newMemo()"><span>＋</span>{{ view === 'todos' ? '添加事项' : '新建备忘录' }}</button></header>
      <div v-if="error" class="error-banner">{{ error }} <button @click="load">重试</button></div>

      <section v-if="view === 'todos'" class="todo-layout">
        <div class="todo-main">
          <form class="quick-add" @submit.prevent="addTodo"><span class="plus-circle">＋</span><input id="todo-input" v-model="todoDraft" placeholder="写下下一件要完成的事..." autocomplete="off" /><button type="submit">添加</button></form>
          <div class="section-heading"><div><h2>航海日志</h2><span>{{ remaining }} 件未完成 · {{ completed }} 件已完成</span></div><span class="date-stamp">{{ new Date().toLocaleDateString('zh-CN', { month: 'long', day: 'numeric' }) }}</span></div>
          <div v-if="loading" class="empty-state">正在读取航海日志...</div>
          <div v-else-if="!todos.length" class="empty-state"><span class="empty-compass">✦</span><strong>海面很平静</strong><p>添加第一件待办，让今天开始有方向。</p></div>
          <ul v-else class="todo-list">
            <li v-for="todo in todos" :key="todo.id" :class="{ done: !!todo.completed }" class="todo-item">
              <button class="check" :aria-label="todo.completed ? '标记未完成' : '标记完成'" @click="toggleTodo(todo)">{{ todo.completed ? '✓' : '' }}</button>
              <input v-if="editingTodo === todo.id" v-model="editingTodoText" class="edit-input" @keyup.enter="saveTodoEdit(todo)" @keyup.esc="editingTodo = null" @blur="saveTodoEdit(todo)" />
              <span v-else class="todo-title" @dblclick="beginTodoEdit(todo)">{{ todo.title }}</span>
              <span class="todo-time">{{ formatDate(todo.updated_at) }}</span>
              <div class="item-actions"><button aria-label="上移" title="上移" @click.stop="moveTodo(todo.id, -1)">↑</button><button aria-label="下移" title="下移" @click.stop="moveTodo(todo.id, 1)">↓</button><button aria-label="编辑" title="编辑" @click.stop="beginTodoEdit(todo)">✎</button><button aria-label="删除" title="删除" @click.stop="removeTodo(todo.id)">×</button></div>
            </li>
          </ul>
        </div>
        <aside class="quote-card"><div class="quote-mark">“</div><p>我把这一刀，<br />留给明天。</p><span>— Red Hair</span><div class="card-wave"></div></aside>
      </section>

      <section v-else class="memo-layout">
        <div class="memo-list"><button class="new-memo-link" @click="newMemo">＋ 新建备忘录</button><div v-if="!memos.length" class="memo-empty">还没有备忘录</div><button v-for="memo in memos" :key="memo.id" class="memo-item" :class="{ selected: selectedMemo?.id === memo.id }" @click="selectMemo(memo)"><strong>{{ memo.title }}</strong><span>{{ memo.body || '暂无内容' }}</span><small>{{ formatDate(memo.updated_at) }}</small></button></div>
        <div class="memo-editor"><div class="editor-top"><span>{{ selectedMemo ? 'EDITING LOG' : 'NEW LOG' }}</span><button v-if="selectedMemo" class="delete-link" @click="removeMemo(selectedMemo.id)">删除</button></div><input v-model="memoTitle" class="memo-title" placeholder="备忘录标题" /><textarea v-model="memoBody" placeholder="记录此刻的想法、灵感或航海坐标..." /><div class="editor-bottom"><small>自动保存需手动确认</small><button class="primary-button" @click="saveMemo">保存备忘录</button></div></div>
      </section>
    </main>
  </div>
</template>

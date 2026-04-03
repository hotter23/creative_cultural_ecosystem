/// <reference types="vite/client" />

declare module '*.vue' {
  import type { DefineComponent } from 'vue'
  const component: DefineComponent<{}, {}, any>
  export default component
}

// 解决 @wangeditor/editor-for-vue 类型声明问题
declare module '@wangeditor/editor-for-vue' {
  import type { DefineComponent } from 'vue'
  import type { IDomEditor, IEditorConfig, IToolbarConfig } from '@wangeditor/editor'

  export const Editor: DefineComponent<{
    defaultConfig?: Partial<IEditorConfig>
    mode?: 'default' | 'simple'
    modelValue?: string
    onCreated?: (editor: IDomEditor) => void
  }>

  export const Toolbar: DefineComponent<{
    editor: any
    defaultConfig?: Partial<IToolbarConfig>
    mode?: 'default' | 'simple'
  }>
}

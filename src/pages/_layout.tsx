import { useTranslation } from 'react-i18next'
import dayjs from 'dayjs'
import relativeTime from 'dayjs/plugin/relativeTime'
import { LayoutItem } from '@/components/LayoutItem'
import { navItems } from './_routers'
import {
  DndContext,
  KeyboardSensor,
  PointerSensor,
  closestCenter,
  useSensor,
  useSensors,
} from '@dnd-kit/core'
import {
  SortableContext,
  sortableKeyboardCoordinates,
  useSortable,
} from '@dnd-kit/sortable'
import { CSS } from '@dnd-kit/utilities'
import type { CSSProperties } from 'react'
import getSystem from '@/utils/get-system'
import { useThemeMode } from '@/services/states'
import { useCustomTheme } from './_layout/hooks/UseCustomTheme'
import { useVerge } from '@/hooks/UseVerge'

export const portableFlag = false

type NavItem = (typeof navItems)[number]

type MenuContextPosition = { top: number; left: number }

interface SortableNavMenuItemProps {
  item: NavItem
  label: string
}

const SortableNavMenuItem = ({ item, label }: SortableNavMenuItemProps) => {
  const {
    attributes,
    listeners,
    setNodeRef,
    transform,
    transition,
    isDragging,
  } = useSortable({
    id: item.path,
  })

  const style: CSSProperties = {
    transform: CSS.Transform.toString(transform),
    transition,
  }

  if (isDragging) {
    style.zIndex = 100
  }
  return (
    <LayoutItem
      to={item.path}
      icon={item.icon}
      sortable={{
        setNodeRef,
        attributes,
        listeners,
        style,
      }}
    >
      {label}
    </LayoutItem>
  )
}

// 为 dayjs 添加相对时间插件，使其支持“几分钟前”等格式化功能
dayjs.extend(relativeTime)

const OS = getSystem()


const Layout = () => {
  const mode = useThemeMode()
  const isDark = mode !== 'light'
  const {t} = useTranslation()
  const {theme} = useCustomTheme()
  const {verge, mutateVerge, patchVerge} = useVerge()
  const {language} = verge ?? {}
  const navCollapsed = verge?.collapse_navbar ?? false
  const {switchLanguage} = useI18n()






  return (
    <div>
      <div>
        {navItems.map((item) => (
          <SortableNavMenuItem key={item.path} item={item} label={item.label} />
        ))}
      </div>
    </div>
  )
}


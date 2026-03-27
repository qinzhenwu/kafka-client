import {
  Server,
  MessageSquare,
  Download,
  Play,
  Square,
  Trash2,
  Eye,
  Plus,
  Pencil,
  RefreshCw,
  Search,
  Send,
  Mail,
  Clipboard,
  Moon,
  Sun,
  AlertTriangle,
  Circle,
  Radio,
  X,
  RotateCcw
} from 'lucide-vue-next'
import TopicIcon from '@/components/icons/TopicIcon.vue'
import ConsumerGroupIcon from '@/components/icons/ConsumerGroupIcon.vue'

export const icons = {
  topic: TopicIcon,
  consumerGroup: ConsumerGroupIcon,
  cluster: Server,
  message: MessageSquare,
  consume: Download,
  streamStart: Play,
  streamStop: Square,
  clear: Trash2,
  view: Eye,
  add: Plus,
  edit: Pencil,
  delete: Trash2,
  refresh: RefreshCw,
  search: Search,
  send: Send,
  mail: Mail,
  clipboard: Clipboard,
  themeDark: Moon,
  themeLight: Sun,
  warning: AlertTriangle,
  connected: Circle,
  disconnected: Circle,
  streaming: Radio,
  close: X,
  reset: RotateCcw
}

export type IconName = keyof typeof icons
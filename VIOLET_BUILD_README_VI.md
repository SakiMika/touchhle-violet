# Source touchHLE — bản vá thử nghiệm Ragnarok Violet V2

## Gói này có gì?

- Source touchHLE tại revision `ca1de89c578e79746f33c1332e28efd106c334ba`.
- Đã áp dụng các bản vá UIGraphics, NSLocale, NSDateFormatter và CALayer.
- File `VIOLET_PATCH_VERSION.txt` đánh dấu gói V2.
- Đã kèm source bốn dependency native: SDL, dynarmic, openal-soft và stb,
  ở đúng revision upstream yêu cầu. Xem `VENDORED_DEPENDENCIES.txt`.
- Dependency native được đưa vào dạng file/thư mục thông thường để tạo repo mới.
  Không còn Git submodule ở cấp repo; không cần chạy git submodule update.
- Chỉ có một workflow đang hoạt động: `.github/workflows/build-violet-windows.yml`.
- Workflow build trực tiếp commit của repo mới bằng actions/checkout, không
  clone lại upstream rồi checkout nhầm commit của repo bạn.
- Boost, CMake, Rust và Cargo dependencies sẽ được tải khi chạy Actions;
  đây không phải gói build offline. Không bao gồm IPA hay dữ liệu lưu game.

## Tạo repo mới bằng GitHub Desktop (khuyên dùng)

1. Giải nén toàn bộ ZIP; không chỉ chép workflow hoặc file patch.
2. Cài và đăng nhập GitHub Desktop: https://desktop.github.com/
3. Trong GitHub Desktop, chọn File > New repository.
4. Đặt tên, ví dụ `touchhle-violet`, chọn nơi lưu rồi Create repository.
5. Chọn Repository > Show in Explorer.
6. Chép TẤT CẢ nội dung thư mục source trong ZIP vào thư mục repo vừa tạo.
   Nếu cần, cho phép ghi đè README/.gitignore mặc định. Không chép nguyên
   thư mục bọc bên ngoài thành thư mục con của repo.
7. Đảm bảo nhìn thấy các thư mục `.github`, `src`, `vendor` và file
   `Cargo.toml` ngay ở gốc repo, cạnh thư mục `.git` do Desktop tạo.
8. Quay lại GitHub Desktop, ghi Summary, ví dụ `Add patched touchHLE source`,
   chọn Commit to main, sau đó Publish repository / Push origin.
9. Có thể chọn repo Private để dùng cá nhân; Actions sử dụng hạn mức tài khoản.

Không upload nguyên file ZIP vào repo rồi bấm chạy: GitHub Actions cần
source ĐÃ GIẢI NÉN ở thư mục gốc. Không nên dùng web Upload files cho toàn
source vì gói có hàng nghìn file và trình duyệt có giới hạn upload.

## Cách khác: đẩy bằng Git

Tạo một repository RỖNG trên github.com/new: không chọn tạo README,
.gitignore hay LICENSE từ giao diện vì gói này đã có sẵn.

Mở terminal trong thư mục source đã giải nén rồi chạy từng lệnh:

```bat
git init
git add .
git commit -m "Add patched touchHLE with Windows Actions"
git branch -M main
git remote add origin https://github.com/TEN_GITHUB/TEN_REPO.git
git push -u origin main
```

Thay TEN_GITHUB và TEN_REPO bằng thông tin repo của bạn. Git có thể yêu cầu
đăng nhập GitHub; không nhập mật khẩu hay token vào chat. Nếu Git báo thiếu
user.name/user.email, cấu hình tên/email commit trước rồi chạy lại git commit.

## Chạy build

1. Trên GitHub vào repo > Actions.
2. Cho phép chạy Actions nếu được hỏi.
3. Chọn `Build Violet Windows` > Run workflow > chọn main > Run workflow.
4. Chờ job hoàn tất. Khi xanh, mở lần chạy, kéo xuống Artifacts.
5. Tải `touchHLE-Violet-Windows-x64`, giải nén.
6. Sao lưu executable cũ rồi chép touchHLE.exe mới vào thư mục runtime
   đang dùng. Giữ các tài nguyên, thư mục IPA, cấu hình và dữ liệu save.

Workflow chỉ chạy thủ công, không tự chạy mỗi lần push. Nếu không thấy
workflow, kiểm tra file yml nằm đúng `.github/workflows` trên nhánh mặc định.
Sau khi sửa workflow và commit, bấm Run workflow để tạo LẦN CHẠY MỚI;
re-run lần cũ có thể vẫn sử dụng nội dung workflow cũ.

Nếu build đỏ, mở bước bị lỗi và gửi log hoặc Download log archive để kiểm tra.

## Cập nhật repo đã có từ gói V1

Nếu repo được tạo từ gói full source V1, giải nén gói UPDATE V2 và chép toàn bộ
nội dung vào GỐC repo, cho phép ghi đè, rồi commit và push. Không chép file ZIP
nguyên vào repo, không chạy git apply: file patch chỉ để xem thay đổi vì source
đã được vá sẵn. Gói UPDATE không dành cho checkout upstream bất kỳ.

Nếu tạo repo mới, dùng gói FULL SOURCE V2, không chỉ dùng gói UPDATE.
Workflow vẫn tên `Build Violet Windows`; phải tạo lần chạy MỚI sau khi push.
Artifact chứa `VIOLET_PATCH_VERSION.txt` ghi `Violet experimental patch V2`.

## Phạm vi bản vá và kiểm chứng V2

- UIGraphics: context RGBA8 thật, snapshot ảnh, context lồng nhau và giữ/nhả bộ nhớ.
- NSLocale: nhận identifier như en_US thay vì assert chỉ cho mã hai ký tự;
  phân tách ngôn ngữ/vùng, giữ identifier gốc và sửa hằng LanguageCode.
- NSDateFormatter: thêm setLocale:/locale và định dạng số, bao gồm giờ 12h `hh`.
- CALayer: thêm renderInContext: để vẽ nền, contents, delegate UIKit và sublayers
  vào context; không phải stub bỏ qua lời gọi. Giữ/khôi phục graphics state.
- Workflow build chính source repo hiện tại, không checkout SHA của repo khác.
  Build release kèm debug info và chép PDB nếu compiler tạo ra, giúp chẩn đoán
  native crash khi cần.

Đã biên dịch bản debug Linux và chạy IPA người dùng cung cấp bằng Mesa software
OpenGL + gles1_on_gl2: qua title, menu, chọn New Game/slot, đoạn giới thiệu và
vào cảnh mở đầu trong nhà với nhân vật/hội thoại. Đây không phải kiểm thử toàn game.
Đã chạy 33 unit test thật (`cargo test --locked --lib`) và 7 test độc lập của bản vá,
đều đạt. Xem `VIOLET_V2_TEST_REPORT.txt`.

CHƯA chạy GitHub Actions/Windows binary thực tế. Lỗi Windows native
0xC0000005 chưa được tái hiện trên Linux; không khẳng định V2 đã sửa lỗi đó.
Không xác nhận âm thanh: máy thử dùng OpenAL null backend. Vẫn có cảnh báo
Objective-C reference count và các API TODO của upstream.

Giới hạn: CALayer chưa xử lý group opacity, clipping, rounded corners hoặc
presentation layer; NSDateFormatter chưa hỗ trợ đầy đủ locale/calendar/timezone
và tên tháng/ngày. NSLocale chỉ phân tách identifier thông dụng, không phải ICU.
UIGraphics chưa có BeginImageContextWithOptions, stack chưa tách theo thread,
các giới hạn UIImage/renderer khác vẫn tồn tại. Hãy sao lưu save trước khi thử.

Đây là bản sửa thử nghiệm có hỗ trợ AI cho repo riêng, không phải bản release
chính thức. CONTRIBUTING.md của upstream cấm đóng góp mã tạo bằng AI;
không coi gói này là mã đã được upstream chấp thuận hoặc gửi upstream như mã tự viết.
Giữ nguyên LICENSE, thông báo bản quyền và license của các thư viện đính kèm.
Nếu phân phối binary, tuân thủ nghĩa vụ license và cung cấp source tương ứng.

## Tài liệu

- Hướng dẫn build upstream kèm theo: `dev-docs/building.md`.
- Hướng dẫn cũ về Git submodule áp dụng cho checkout upstream, không phải
  snapshot đã gộp dependency trong gói này.
- Hệ tọa độ UIKit / Core Graphics:
  https://developer.apple.com/library/archive/documentation/2DDrawing/Conceptual/DrawingPrintingiOS/GraphicsDrawingOverview/GraphicsDrawingOverview.html

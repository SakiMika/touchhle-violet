# Source touchHLE đã vá API UIGraphics cho Ragnarok Violet

## Gói này có gì?

- Source touchHLE tại revision `ca1de89c578e79746f33c1332e28efd106c334ba`.
- Đã áp dụng bản vá trong `src/frameworks/uikit/ui_graphics.rs`.
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

## Phạm vi bản vá và kiểm chứng

Log gốc báo `Call to unimplemented function _UIGraphicsBeginImageContext`.
Bản vá bổ sung UIGraphicsBeginImageContext, UIGraphicsGetImageFromCurrentImageContext
và UIGraphicsEndImageContext bằng bitmap thật, có giữ/nhả bộ nhớ và context lồng nhau.
Không cần áp dụng lại file patch: mã nguồn trong gói ĐÃ ĐƯỢC VÁ.

Bảy bài test độc lập đã qua trên Linux với test doubles cho Core Graphics/
Objective-C. Test này không thay thế full cargo test, renderer thật hoặc chạy game.
Workflow cũng chạy lại bảy test đó trước khi build executable trên Windows.

Chưa chạy workflow này thực tế trên GitHub, chưa có Windows binary được xác minh,
chưa chạy lại game do chưa có IPA. Build thành công không chứng minh game không
còn crash: các API thiếu hoặc lỗi vẽ khác có thể xuất hiện tiếp. Chưa sửa giới hạn
UIImage/renderer sẵn có, chưa hỗ trợ UIGraphicsBeginImageContextWithOptions,
context stack vẫn dùng state chung như source ban đầu.

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
